use axum::{
    extract::ws::Message,
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common::Scores;
use crate::common::SocketMessage;
use crate::common::CONFIG;
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

/// Holds the client-server connections between two peers.
struct Connection {
    left: WebSocket,
    right: WebSocket,
}

impl Connection {
    pub fn new(left: WebSocket, right: WebSocket) -> Self {
        Self { left, right }
    }

    /// Handles sending messages from one peer to another.
    pub async fn run(self) {
        eprintln!("communication starting between a pair");
        let msg = "connected to peer!".to_string();

        let (mut left_tx, mut left_rx) = self.left.split();
        let (mut right_tx, mut right_rx) = self.right.split();

        let _ = right_tx.send(SocketMessage::info_msg(msg.clone())).await;
        let _ = left_tx.send(SocketMessage::info_msg(msg)).await;

        loop {
            tokio::select! {
                Some(Ok(msg)) = right_rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            let _ = left_tx.send(SocketMessage::info_msg("Peer disconnected".to_string())).await;
                            break;
                        },
                        Message::Text(msg) => {
                            eprintln!("right->left: {}", &msg);
                            if left_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                eprintln!("Failed to send message to right");
                                break;
                            }
                        },
                        _ => {}
                    }
                }
                Some(Ok(msg)) = left_rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            let _ = right_tx.send(SocketMessage::info_msg("Peer disconnected".to_string())).await;
                            break;
                        },
                        Message::Text(msg) => {
                            eprintln!("left->right: {}", &msg);
                            if right_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                eprintln!("Failed to send message to right");
                                break;
                            }
                        },
                        _ => {}
                    }
                }
                else => {
                    let _ = left_tx.send(SocketMessage::user_msg("unexpected error occured".to_string())).await;
                    let _ = right_tx.send(SocketMessage::user_msg("unexpected error occured".to_string())).await;
                    break;
                }
            }
        }
    }
}

struct WaitingUser {
    score: Scores,
    socket: WebSocket,
}

/// If 2 or more users are present, it'll pop the longest-waiting user along with
/// another user who has the closest personality.
fn pair_pop(users: &mut Vec<WaitingUser>) -> Option<(WaitingUser, WaitingUser)> {
    let len = users.len();
    eprintln!("users waiting {}", len);
    if len < 2 {
        return None;
    }

    // prioritize the user who waited the longest.
    let left = users.remove(0);

    let mut right_index = 0;
    let mut closest = f32::MAX;

    for (index, user) in users.iter().enumerate() {
        let diff = left.score.distance(&user.score);
        if diff < closest {
            closest = diff;
            right_index = index;
        }
    }

    let right = users.remove(right_index);

    eprintln!("two users paired up!");
    Some((left, right))
}

#[derive(Default, Clone)]
struct State {
    // Users waiting to be matched with a peer.
    users_waiting: Arc<Mutex<Vec<WaitingUser>>>,
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    fn queue(&self, score: Scores, socket: WebSocket) {
        eprintln!("user queued ");
        let user = WaitingUser { score, socket };
        self.users_waiting.lock().unwrap().push(user);
    }

    async fn start_pairing(&self) {
        eprintln!("pairing started");
        let users = Arc::clone(&self.users_waiting);
        tokio::spawn(async move {
            loop {
                {
                    let mut lock = users.lock().unwrap();
                    while let Some((left, right)) = pair_pop(&mut lock) {
                        tokio::spawn(async move {
                            Connection::new(left.socket, right.socket).run().await;
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
    Path(scores): Path<String>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    eprintln!("pair handling!");
    let scores: Scores = scores.parse().unwrap();
    ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            let state = state.clone();
            state.queue(scores, socket);
        }
    })
}

pub async fn run() {
    let state = State::new();
    state.start_pairing().await;

    eprintln!("starting server ");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/pair/:scores", get(pair_handler))
        .layer(cors)
        .layer(Extension(Arc::new(state)));

    let addr = "0.0.0.0:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
