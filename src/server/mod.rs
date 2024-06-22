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
#[cfg(not(test))]
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

#[cfg(test)]
async fn queue(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    serde_json::to_string(&state.waiting_users.user_ids().await).unwrap()
}

#[cfg(test)]
async fn cons(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    serde_json::to_string(&state.connections.pairs().await).unwrap()
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

pub async fn run(port: u16) {
    #[cfg(not(test))]
    let file_appender = tracing_appender::rolling::daily("log", "ocean-chat.log");
    #[cfg(not(test))]
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    #[cfg(not(test))]
    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "ocean_chat=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

    #[cfg(not(test))]
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
    let router = router.route("/queue", get(queue)).route("/cons", get(cons));

    let app = router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(Arc::new(state)));

    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
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

    struct TestFramework {
        port: u16,
    }

    impl TestFramework {
        fn new(port: u16) -> Self {
            Self::start_server(port);
            Self { port }
        }

        async fn get_pairs(&self) -> Vec<(String, String)> {
            let url = format!("http://127.0.0.1:{}/cons", self.port);
            let response = reqwest::get(url).await.unwrap().text().await.unwrap();
            serde_json::from_str(&response).unwrap()
        }

        async fn get_waiting_users(&self) -> Vec<String> {
            let url = format!("http://127.0.0.1:{}/queue", self.port);
            let response = reqwest::get(url).await.unwrap().text().await.unwrap();
            serde_json::from_str(&response).unwrap()
        }

        async fn queue_user(&self, id: impl Into<String>) -> WebSocket {
            self.queue_user_with_score(Scores::mid(), id).await
        }

        async fn queue_user_with_score(&self, s: Scores, id: impl Into<String>) -> WebSocket {
            let id = id.into();
            let port = self.port;

            tokio::spawn(async move {
                let f = format!("ws://127.0.0.1:{}", port);

                let url = format!("{}/pair/{}/{}", f, s, id);
                let url = Url::parse(&url);
                let url = url.unwrap();
                let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
                ws_stream
            })
            .await
            .unwrap()
        }

        async fn assert_pair(&self, left: &str, right: &str) {
            let mut pair_found = false;

            for (xleft, xright) in self.get_pairs().await {
                if (xleft == left && xright == right) || (xleft == right && xright == left) {
                    pair_found = true;
                }
            }

            assert!(pair_found);
        }

        async fn assert_waiting_users(&self, expected: Vec<&str>) {
            let waiting_users = self.get_waiting_users().await;
            assert_eq!(expected.len(), waiting_users.len());

            for user in expected {
                assert!(waiting_users.contains(&user.to_string()));
            }
        }

        fn start_server(port: u16) {
            tokio::spawn(async move {
                run(port).await;
            });
        }
    }

    #[tokio::test]
    async fn test_queue_user() {
        let tfw = TestFramework::new(3000);

        let id = "heythere";
        let _ws = tfw.queue_user(id).await;

        std::thread::sleep(std::time::Duration::from_secs(1));

        tfw.assert_waiting_users(vec![id]).await;
    }

    #[tokio::test]
    async fn test_connect_pair() {
        let tfw = TestFramework::new(3001);
        let id = "foo";
        let id2 = "bar";

        let _ws = tfw.queue_user(id).await;
        let _ws2 = tfw.queue_user(id2).await;
        std::thread::sleep(std::time::Duration::from_secs(3));

        // They should have paired up and thus nobody in waiting quue.
        tfw.assert_waiting_users(vec![]).await;
        tfw.assert_pair(id, id2).await;
    }
}
