use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common;

use crate::server::ConnectionManager;
use common::Scores;
use common::SocketMessage;
use common::CONFIG;
use std::sync::Arc;
use std::time::SystemTime;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;

mod connection;
mod waiting_users;

use connection::*;
use waiting_users::*;

pub struct User {
    pub scores: Scores,
    pub socket: WebSocket,
    pub id: String,
    pub con_time: SystemTime,
}

impl User {
    async fn ping(&mut self) -> bool {
        let ping_timeout = tokio::time::Duration::from_millis(2000);
        if self.socket.send(SocketMessage::ping()).await.is_err() {
            return false;
        }

        while let Ok(Some(Ok(Message::Binary(msg)))) =
            tokio::time::timeout(ping_timeout, self.socket.recv()).await
        {
            if let Ok(SocketMessage::Pong) = serde_json::from_slice(&msg) {
                return true;
            }
        }

        false
    }

    async fn drain_socket(&mut self) {
        let drain_timeout = tokio::time::Duration::from_millis(100);
        while let Ok(Some(_)) = tokio::time::timeout(drain_timeout, self.socket.recv()).await {}
    }
}

#[derive(Default, Clone)]
struct State {
    // Users waiting to be matched with a peer.
    waiting_users: WaitingUsers,
    connections: ConnectionManager,
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    async fn queue(&self, scores: Scores, id: String, socket: WebSocket) {
        tracing::info!("user queued ");
        let con_time = SystemTime::now();
        let user = User {
            scores,
            socket,
            id,
            con_time,
        };
        self.waiting_users.queue(user).await;
    }

    fn stats_printer(&self) {
        let waits = self.waiting_users.clone();
        let cons = self.connections.clone();

        tokio::spawn(async move {
            loop {
                let stat = {
                    let waiting = waits.len().await;
                    let connected = cons.connected_users_qty().await;
                    (waiting, connected)
                };

                let (waiting, connected) = stat;
                tracing::info!("users waiting: {}, connected users: {}", waiting, connected);

                tokio::time::sleep(std::time::Duration::from_secs(600)).await;
            }
        });
    }

    async fn start_pairing(&self) {
        tracing::info!("pairing started");
        let users = self.waiting_users.clone();
        let connections = self.connections.clone();
        tokio::spawn(async move {
            loop {
                {
                    while let Some((mut left, mut right)) = users.pop_pair().await {
                        let users = users.clone();
                        let connections = connections.clone();
                        tokio::spawn(async move {
                            let left_pinged = left.ping().await;
                            let right_pinged = right.ping().await;

                            // If they're the same user, put the newest connection back in queue
                            // (if pingable).
                            if right.id == left.id {
                                if right.con_time > left.con_time && right_pinged {
                                    users.queue(right).await;
                                    let _ = left.socket.close().await;
                                } else if right.con_time < left.con_time && left_pinged {
                                    users.queue(left).await;
                                    let _ = right.socket.close().await;
                                }
                                return;
                            }

                            match (left_pinged, right_pinged) {
                                (true, true) => {
                                    tracing::info!("ping successful");
                                    connections.connect(left, right).await;
                                }
                                (true, false) => {
                                    tracing::error!("failed to ping right");
                                    users.queue(left).await;
                                }
                                (false, true) => {
                                    tracing::error!("failed to ping left");
                                    users.queue(right).await;
                                }
                                (false, false) => {
                                    tracing::error!("failed to ping both right and left");
                                }
                            }
                        });
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(
                    CONFIG.pair_interval_millis,
                ))
                .await;
            }
        });
    }
}

async fn pair_handler(
    Path((scores, id)): Path<(String, String)>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    tracing::info!("pair handling!");
    let scores: Scores = scores.parse().unwrap();
    ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            let state = state.clone();
            state.queue(scores, id, socket).await;
        }
    })
}

pub async fn run() {
    let file_appender = tracing_appender::rolling::daily("log", "ocean-chat.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "ocean_chat=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");

    tracing::info!("starting server ");
    let state = State::new();
    state.start_pairing().await;
    state.stats_printer();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/pair/:scores/:id", get(pair_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(Arc::new(state)));

    let addr = "0.0.0.0:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
