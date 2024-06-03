use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common;

use crate::server::connection::Connection;
use crate::server::waiting_users::WaitingUsers;
use common::Scores;
use common::SocketMessage;
use common::CONFIG;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod connection;
mod waiting_users;

pub struct User {
    pub scores: Scores,
    pub socket: WebSocket,
}

impl User {
    async fn ping(&mut self) -> bool {
        let ping_timeout = tokio::time::Duration::from_millis(500);
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
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    async fn queue(&self, scores: Scores, socket: WebSocket) {
        tracing::info!("user queued ");
        let user = User { scores, socket };
        self.waiting_users.queue(user).await;
    }

    async fn start_pairing(&self) {
        tracing::info!("pairing started");
        let users = self.waiting_users.clone();
        tokio::spawn(async move {
            loop {
                {
                    while let Some((mut left, mut right)) = users.pop_pair().await {
                        let left_pinged = left.ping().await;
                        let right_pinged = right.ping().await;

                        match (left_pinged, right_pinged) {
                            (true, true) => {
                                tracing::error!("ping successful");
                                tokio::spawn(async move {
                                    Connection::new(left, right).run().await;
                                });
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
    Path(scores): Path<String>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    tracing::info!("pair handling!");
    let scores: Scores = scores.parse().unwrap();
    ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            let state = state.clone();
            state.queue(scores, socket).await;
        }
    })
}

pub async fn run() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "ocean_chat=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let tracing_layer = TraceLayer::new_for_http();

    tracing::info!("starting server ");
    let state = State::new();
    state.start_pairing().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/pair/:scores", get(pair_handler))
        .layer(cors)
        .layer(tracing_layer)
        .layer(Extension(Arc::new(state)));

    let addr = "0.0.0.0:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
