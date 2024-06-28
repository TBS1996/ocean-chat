use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common;

use crate::common::ChangeState;
use crate::server::ConnectionManager;
use common::Scores;
use common::CONFIG;
use std::sync::Arc;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod connection;
mod user;
mod waiting_users;

use connection::*;
use user::*;
use waiting_users::*;

pub struct UpMsg {
    pub id: String,
    pub msg: MsgStuff,
}

pub enum MsgStuff {
    StateChange(ChangeState),
}

#[derive(Default, Clone)]
struct IdleUsers(Arc<Mutex<HashMap<String, User>>>);

use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone)]
struct State {
    idle_users: IdleUsers,
    waiting_users: WaitingUsers,
    connections: ConnectionManager,
    tx: mpsc::Sender<UpMsg>,
}

impl State {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(32);

        let selv = Self {
            tx,
            idle_users: Default::default(),
            waiting_users: Default::default(),
            connections: Default::default(),
        };

        let s = selv.clone();

        tokio::spawn(async move {
            s.receive_stuff(rx).await;
        });

        selv
    }

    async fn receive_stuff(self, mut rx: mpsc::Receiver<UpMsg>) {
        loop {
            let Some(x) = rx.recv().await else {
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            };

            let id = x.id;

            match x.msg {
                MsgStuff::StateChange(state) => {
                    self.change_state(state, id).await;
                }
            }
        }
    }

    async fn change_state(&self, new_state: ChangeState, id: String) {
        let Some(user) = self.take_user(id.clone()).await else {
            return;
        };

        match new_state {
            ChangeState::Idle => {
                self.idle_users.0.lock().await.insert(id, user);
            }
            ChangeState::Waiting => {
                self.waiting_users.queue(user).await;
            }
        }
    }

    /// Extract user from any state
    async fn take_user(&self, id: String) -> Option<User> {
        if let Some(user) = self.idle_users.0.lock().await.remove(&id) {
            return Some(user);
        }

        if let Some(user) = self.waiting_users.take(&id).await {
            return Some(user);
        }

        if let Some((left, right)) = self.connections.take(&id).await {
            if left.id == id {
                return Some(left);
            }

            if right.id == id {
                return Some(right);
            }
        }

        None
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    async fn queue(&self, scores: Scores, id: String, socket: WebSocket) {
        let tx = self.tx.clone();
        let user = User::new(scores, id, socket, tx);
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

async fn user_status(
    Path(id): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    use crate::common::UserStatus;
    tracing::info!("user status!: {}", &id);

    let status = if state.waiting_users.contains(&id).await {
        state.connections.clear_user(&id).await;

        UserStatus::Waiting
    } else if state.connections.contains(&id).await {
        UserStatus::Connected
    } else {
        UserStatus::Disconnected
    };

    serde_json::to_string(&status).unwrap()
}

pub async fn run(port: u16) {
    #[cfg(not(test))]
    {
        use tracing_subscriber::layer::SubscriberExt;

        let file_appender = tracing_appender::rolling::daily("log", "ocean-chat.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "warn,ocean_chat=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
    }

    tracing::info!("starting server ");
    let state = State::new();
    state.start_pairing().await;
    state.stats_printer();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/pair/:scores/:id", get(pair_handler))
        .route("/status/:id", get(user_status));

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
    use crate::common::SocketMessage;
    use crate::common::UserStatus;
    use futures_util::SinkExt;
    use futures_util::TryStreamExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use url::Url;

    type WebSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

    fn start_server(port: u16) {
        tokio::spawn(async move {
            run(port).await;
        });
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    struct TestSocket {
        port: u16,
        id: String,
        ws: WebSocket,
    }

    impl TestSocket {
        async fn new(id: impl Into<String>, port: u16) -> Self {
            let id = id.into();
            let ws = Self::queue_user(&id, port).await;

            Self { ws, port, id }
        }

        async fn get_message(&mut self) -> Option<SocketMessage> {
            let msg = self.ws.try_next().await.ok()??;
            Some(serde_json::from_str(&msg.to_string()).unwrap())
        }

        async fn is_closed(&mut self) -> bool {
            let s = SocketMessage::Ping.to_string().into_bytes();
            let res = self
                .ws
                .send(tokio_tungstenite::tungstenite::Message::Binary(s))
                .await
                .unwrap();

            dbg!(res);

            self.get_message().await.is_none()
        }

        async fn get_status(&self) -> UserStatus {
            let url = format!("http://127.0.0.1:{}/status/{}", self.port, &self.id);
            let response = reqwest::get(url).await.unwrap().text().await.unwrap();
            serde_json::from_str(&response).unwrap()
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

        async fn queue_user(id: impl Into<String>, port: u16) -> WebSocket {
            Self::queue_user_with_score(Scores::mid(), id, port).await
        }

        async fn queue_user_with_score(s: Scores, id: impl Into<String>, port: u16) -> WebSocket {
            let id = id.into();

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
    }

    #[tokio::test]
    async fn test_queue_user() {
        let port = 3000;

        start_server(port);

        let id = "heythere";
        let ws = TestSocket::new(id, port).await;
        std::thread::sleep(std::time::Duration::from_secs(1));

        ws.assert_waiting_users(vec![id]).await;
    }

    #[tokio::test]
    async fn test_connect_pair() {
        let port = 3001;
        start_server(port);

        let id = "foo";
        let id2 = "bar";

        let ws = TestSocket::new(id, port).await;
        assert_eq!(ws.get_status().await, UserStatus::Waiting);
        let ws2 = TestSocket::new(id2, port).await;
        std::thread::sleep(std::time::Duration::from_secs(3));
        assert_eq!(ws.get_status().await, UserStatus::Connected);
        assert_eq!(ws2.get_status().await, UserStatus::Connected);
    }

    #[tokio::test]
    async fn test_close_connection_when_same_connects() {
        let port = 3002;
        start_server(port);
        let id = "foo";

        let mut ws = TestSocket::new(id, port).await;
        assert!(!ws.is_closed().await);
        let mut other_ws = TestSocket::new(id, port).await;
        std::thread::sleep(std::time::Duration::from_secs(1));
        dbg!(ws.get_waiting_users().await);
        assert!(ws.is_closed().await);
        assert!(other_ws.is_closed().await);
    }
}
