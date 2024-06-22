use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common;

use crate::server::ConnectionManager;
use common::Scores;
use common::CONFIG;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;

mod connection;
mod user;
mod waiting_users;

use connection::*;
use user::*;
use waiting_users::*;

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
        let user = User::new(scores, id, socket);
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
                    while let Some((left, right)) = users.pop_pair().await {
                        let connections = connections.clone();
                        tokio::spawn(async move {
                            connections.connect(left, right).await;
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

async fn queue(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    serde_json::to_string(&state.waiting_users.user_ids().await).unwrap()
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

    let router = Router::new().route("/pair/:scores/:id", get(pair_handler));

    #[cfg(test)]
    let router = router.route("/queue", get(queue));

    let app = router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(Arc::new(state)));

    let addr = "0.0.0.0:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use url::Url;

    type WebSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

    fn start_server() {
        tokio::spawn(async move {
            run().await;
        });
    }

    async fn get_waiting_users() -> Vec<String> {
        let response = reqwest::get("http://127.0.0.1:3000/queue")
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        serde_json::from_str(&response).unwrap()
    }

    async fn queue_user(s: Scores, id: impl Into<String>) -> WebSocket {
        let id = id.into();
        tokio::spawn(async move {
            let url = format!("{}/pair/{}/{}", CONFIG.server_address(), s, id);
            let url = Url::parse(&url);
            let url = url.unwrap();
            let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
            ws_stream
        })
        .await
        .unwrap()
    }

    async fn assert_waiting_users(expected: Vec<&str>) {
        let waiting_users = get_waiting_users().await;
        assert_eq!(expected.len(), waiting_users.len());

        for user in expected {
            assert!(waiting_users.contains(&user.to_string()));
        }
    }

    #[tokio::test]
    async fn test_queue_user() {
        start_server();

        let id = "heythere";
        let _ws = queue_user(Scores::mid(), id).await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        assert_waiting_users(vec![id]).await;
    }
}
