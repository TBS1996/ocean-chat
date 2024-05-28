use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Json},
    routing::get,
    routing::post,
    Router,
};

use crate::common::Scores;
use axum::extract::Path;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

type UserId = Uuid;

const PAIR_WINDOW_MILLIS: u64 = 1000;

static STATE: Lazy<Arc<Mutex<State>>> = Lazy::new(|| {
    let s = State::new();
    Arc::new(Mutex::new(s))
});

/// Bi-directional mapping of all paired up users.
#[derive(Clone, Default)]
struct Pairs {
    inner: HashMap<UserId, UserId>,
}

impl Pairs {
    fn insert(&mut self, a: UserId, b: UserId) {
        self.inner.insert(a, b);
        self.inner.insert(b, a);
    }

    fn remove(&mut self, id: UserId) {
        let peer = self.get(id);
        self.inner.remove(&id);
        self.inner.remove(&peer);
    }

    fn get(&self, id: UserId) -> UserId {
        self.inner.get(&id).unwrap().to_owned()
    }
}

/// Response from server when frontend requests a peer.
#[derive(Serialize)]
struct PairResponse {
    peer_id: String,
}

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
    async fn run(mut self) {
        loop {
            tokio::select! {
                Some(message) = self.left.next() => {
                    match message {
                        Ok(msg) => {
                            if self.right.send(msg).await.is_err() {
                                eprintln!("Failed to send message to right");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from left: {:?}", e);
                            break;
                        }
                    }
                }
                Some(message) = self.right.next() => {
                    match message {
                        Ok(msg) => {
                            if self.left.send(msg).await.is_err() {
                                eprintln!("Failed to send message to left");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from right: {:?}", e);
                            break;
                        }
                    }
                }
                else => break,
            }
        }
    }
}

#[derive(Default)]
struct State {
    // The users who have clicked submit but have not yet been assigned a peer.
    users_waiting: Arc<Mutex<Vec<WaitingUser>>>,
    // Map of all pairs that have yet to establish a connection.
    pairs: Pairs,
    /// There's a brief period where one user has established a connection
    /// but his peer have not.
    cons: HashMap<UserId, WebSocket>,
}

struct WaitingUser {
    id: UserId,
    callback: oneshot::Sender<UserId>,
    score: Scores,
}

/// If 2 or more users are present, it'll pop the longest-waiting user along with
/// another user who has the closest personality.
fn pair_pop(users: &mut Vec<WaitingUser>) -> Option<(WaitingUser, WaitingUser)> {
    if users.len() < 2 {
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

    Some((left, right))
}

impl State {
    fn new() -> Self {
        let s = Self::default();
        s.start_pairing();
        s
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    fn queue(&self, id: UserId, score: Scores) -> oneshot::Receiver<UserId> {
        let (tx, rx) = oneshot::channel();
        let user = WaitingUser {
            id,
            callback: tx,
            score,
        };
        self.users_waiting.lock().unwrap().push(user);
        rx
    }

    fn start_pairing(&self) {
        let users = Arc::clone(&self.users_waiting);

        std::thread::spawn(move || loop {
            {
                let mut lock = users.lock().unwrap();

                while let Some((left, right)) = pair_pop(&mut lock) {
                    left.callback.send(right.id).unwrap();
                    right.callback.send(left.id).unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(PAIR_WINDOW_MILLIS));
        });
    }
}

// Returns the user with the closest personality score to the given scores argument.
pub async fn pair_handler(Json(scores): Json<Scores>) -> impl IntoResponse {
    let user_id = Uuid::new_v4();

    println!("Generated UUID for new user: {}", user_id);

    let peer_id = {
        let state = STATE.lock().unwrap();
        state.queue(user_id, scores)
    };

    let peer_id = peer_id.await.unwrap();
    {
        let mut state = STATE.lock().unwrap();
        state.pairs.insert(user_id, peer_id);
    }
    let peer_id = peer_id.to_string();
    Json(PairResponse { peer_id })
}

/// Sets up a websocket communication with the user and its peer.
///
/// If the peer has already connected, a [`Connection`] is set up and communication can start.
/// If the peer has yet to connect, the caller's websocket is saved and waits for the peer to
/// connect.
pub async fn connect_handler(
    ws: WebSocketUpgrade,
    Path(peer_id): Path<String>,
) -> impl IntoResponse {
    let state = STATE.clone();

    let peer_id: UserId = peer_id.parse().unwrap();
    let caller_id = state.lock().unwrap().pairs.get(peer_id);

    ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            let mut state = state.lock().unwrap();

            match state.cons.remove(&caller_id) {
                None => {
                    state.cons.insert(peer_id, socket);
                }
                Some(peer_socket) => {
                    state.pairs.remove(peer_id);
                    tokio::spawn(async move {
                        Connection::new(peer_socket, socket).run().await;
                    });
                }
            }
        }
    })
}

pub fn run() {
    eprintln!("starting server");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/pair", post(pair_handler))
        .route("/connect/:id", get(connect_handler))
        .layer(cors);

    let addr = "127.0.0.1:3000".parse().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}
