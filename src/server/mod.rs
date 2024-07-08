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
use crate::common::UserStatus;
use crate::server::ConnectionManager;
use common::Scores;
use common::CONFIG;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod connection;
mod user;
mod waiting_users;

use connection::*;
use user::*;
use waiting_users::*;

pub struct StateMessage {
    pub id: UserId,
    pub action: StateAction,
}

impl StateMessage {
    pub fn new(id: UserId, action: StateAction) -> Self {
        Self { id, action }
    }
}

pub enum StateAction {
    StateChange(ChangeState),
    RemoveUser,
    GetStatus(oneshot::Sender<UserStatus>),
}

#[derive(Default, Clone, Debug)]
struct IdleUsers(Arc<Mutex<HashMap<String, User>>>);

impl IdleUsers {
    async fn contains(&self, id: &str) -> bool {
        self.0.lock().await.contains_key(id)
    }

    async fn insert(&self, mut user: User) {
        let _ = user.refresh_status().await;
        let id = user.id.clone();
        self.0.lock().await.insert(id, user);
    }
}

use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct State {
    idle_users: IdleUsers,
    waiting_users: WaitingUsers,
    connections: ConnectionManager,
    tx: mpsc::Sender<StateMessage>,
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

        let state = selv.clone();

        tokio::spawn(async move {
            state.state_message_handler(rx).await;
        });

        selv
    }

    async fn user_status(&self, id: &UserId) -> UserStatus {
        use crate::common::UserStatus;

        let is_waiting = self.waiting_users.contains(&id).await;
        let is_idle = self.idle_users.contains(&id).await;
        let is_connected = self.connections.contains(&id).await;

        match (is_waiting, is_idle, is_connected) {
            (true, false, false) => UserStatus::Waiting,
            (false, true, false) => UserStatus::Idle,
            (false, false, true) => UserStatus::Connected,
            (false, false, false) => UserStatus::Disconnected,
            invalid => {
                tracing::error!("{}: Invalid user status: {:?}", id, invalid);
                self.take_user(id);

                UserStatus::Disconnected
            }
        }
    }

    /// Receive messages from [`User`] that changes the [`State`].
    async fn state_message_handler(self, mut rx: mpsc::Receiver<StateMessage>) {
        loop {
            let Some(msg) = rx.recv().await else {
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            };

            let id = msg.id;

            match msg.action {
                StateAction::GetStatus(tx) => {
                    let status = self.user_status(&id).await;
                    tx.send(status).ok();
                }
                StateAction::StateChange(state) => {
                    self.change_state(state, id).await;
                }
                StateAction::RemoveUser => match self.take_user(id.clone()).await {
                    Some(user) => {
                        tracing::info!("{}: Removed user", user.id);
                    }
                    None => {
                        tracing::error!("{}: Failed to remove user. User not found.", id);
                    }
                },
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
            tracing::info!("{}: user taken from idle queue", &id);
            return Some(user);
        }

        if let Some(user) = self.waiting_users.take(&id).await {
            tracing::info!("{}: user taken from waiting queue", &id);
            return Some(user);
        }

        if let Some((left, right)) = self.connections.take(&id).await {
            if left.id == id {
                tracing::info!("{}: inserting to idle", &right.id);
                self.idle_users.insert(right).await;
                tracing::info!("{}: user taken from connection", &left.id);
                return Some(left);
            }

            if right.id == id {
                tracing::info!("{}: inserting to idle", &left.id);
                self.idle_users.insert(left).await;
                tracing::info!("{}: user taken from connection", &right.id);
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
async fn idle(Extension(state): Extension<Arc<State>>) -> impl IntoResponse {
    let ids: Vec<UserId> = state
        .idle_users
        .0
        .lock()
        .await
        .iter()
        .map(|(_, user)| user.id.clone())
        .collect();
    serde_json::to_string(&ids).unwrap()
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
    let status = state.user_status(&id).await;
    serde_json::to_string(&status).unwrap()
}

pub async fn run(port: u16) {
    //#[cfg(test)]
    #[cfg(not(test))]
    let _guard = {
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
        _guard
    };

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
    let router = router
        .route("/idle", get(idle))
        .route("/queue", get(queue))
        .route("/cons", get(cons));

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
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use url::Url;

    type WebSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

    struct Server {
        port: u16,
    }

    impl Server {
        fn new(port: u16) -> Self {
            tokio::spawn(async move {
                run(port).await;
            });
            std::thread::sleep(std::time::Duration::from_millis(250));

            Self { port }
        }

        async fn connect(&self, id: impl Into<String>) -> TestSocket {
            let id = id.into();
            let ws = self.queue_user(&id).await;
            sleep(0.1).await;

            TestSocket { ws, id }
        }

        async fn get_idle(&self) -> Vec<UserId> {
            let url = format!("http://127.0.0.1:{}/cons", self.port);
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

        async fn empty_state(&self) -> bool {
            self.connected_users().await == 0
        }

        async fn connected_users(&self) -> usize {
            self.get_idle().await.len()
                + self.get_waiting_users().await.len()
                + (self.get_pairs().await.len() * 2)
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

        async fn assert_waiting_users(&self, expected: Vec<&str>) {
            let waiting_users = self.get_waiting_users().await;
            assert_eq!(expected.len(), waiting_users.len());

            for user in expected {
                assert!(waiting_users.contains(&user.to_string()));
            }
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
    }

    struct TestSocket {
        id: String,
        ws: WebSocket,
    }

    impl TestSocket {
        async fn close_connection(&mut self) {
            self.ws.close(None).await.unwrap();
        }

        async fn get_message(&mut self) -> Option<SocketMessage> {
            let msg = tokio::time::timeout(tokio::time::Duration::from_secs(1), self.ws.next())
                .await
                .ok()??
                .ok()?;

            Some(serde_json::from_str(&msg.to_string()).unwrap())
        }

        async fn send_message(&mut self, msg: SocketMessage) {
            self.ws.send(Message::Binary(msg.to_bytes())).await.unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        async fn get_status(&mut self) -> UserStatus {
            self.send_message(SocketMessage::GetStatus).await;
            sleep(0.1).await;

            let mut statuses = vec![];
            while let Some(msg) = self.get_message().await {
                if let SocketMessage::Status(status) = msg {
                    statuses.push(status);
                };
            }

            statuses.last().unwrap().to_owned()
        }

        async fn assert_status(&mut self, status: UserStatus) {
            let current_status = self.get_status().await;
            assert_eq!(current_status, status);
        }
    }

    async fn sleep_pair_interval() {
        let secs = CONFIG.pair_interval_millis as f32 / 1000.;
        sleep(secs + 0.1).await;
    }

    async fn sleep(secs: f32) {
        let millis = (secs * 1000.) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(millis)).await;
    }

    #[tokio::test]
    async fn test_queue_user() {
        let server = Server::new(3000);

        let id = "heythere";
        let _ws = server.connect(id).await;
        sleep(1.0).await;

        server.assert_waiting_users(vec![id]).await;
    }

    #[tokio::test]
    async fn test_connect_pair() {
        let server = Server::new(3001);

        let id = "foo";
        let id2 = "bar";

        let mut ws = server.connect(id).await;
        ws.assert_status(UserStatus::Waiting).await;
        let mut ws2 = server.connect(id2).await;
        sleep_pair_interval().await;
        ws.assert_status(UserStatus::Connected).await;
        ws2.assert_status(UserStatus::Connected).await;
    }

    #[tokio::test]
    async fn test_close_connection_when_same_connects() {
        let server = Server::new(3002);
        let id = "foo";
        let _ws = server.connect(id).await;
        sleep(0.1).await;
        assert_eq!(server.connected_users().await, 1);
        let mut other_ws = server.connect(id).await;
        sleep_pair_interval().await;

        // Only one user meaning the two sockets did not pair up.
        assert_eq!(server.connected_users().await, 1);

        // The second one connected is now waiting.
        other_ws.assert_status(UserStatus::Waiting).await;
    }

    #[tokio::test]
    async fn test_state_change() {
        let server = Server::new(3003);

        // Assert new connections are put in waiting queue.
        let mut ws = server.connect("foo").await;
        ws.assert_status(UserStatus::Waiting).await;

        // Show we can change them to idle.
        ws.send_message(SocketMessage::StateChange(ChangeState::Idle))
            .await;
        ws.assert_status(UserStatus::Idle).await;

        // .. and back to waiting
        ws.send_message(SocketMessage::StateChange(ChangeState::Waiting))
            .await;
        ws.assert_status(UserStatus::Waiting).await;

        sleep(1.).await;
        // If another user then connects, theyre both connected.
        let mut other_ws = server.connect("bar").await;
        sleep(1.).await;
        ws.assert_status(UserStatus::Connected).await;
        other_ws.assert_status(UserStatus::Connected).await;

        // If one goes to waiting, the other is set to idle.
        ws.send_message(SocketMessage::StateChange(ChangeState::Waiting))
            .await;
        sleep(1.).await;

        other_ws.assert_status(UserStatus::Idle).await;
        ws.assert_status(UserStatus::Waiting).await;
    }

    /// Test that other user is set to idle if one close connection.
    #[tokio::test]
    async fn test_close_connection() {
        let server = Server::new(3004);
        let mut aws = server.connect("foo").await;
        let mut bws = server.connect("bar").await;
        sleep(3.).await;

        aws.assert_status(UserStatus::Connected).await;
        bws.assert_status(UserStatus::Connected).await;

        aws.close_connection().await;
        sleep(1.).await;

        bws.assert_status(UserStatus::Idle).await;
    }
}
