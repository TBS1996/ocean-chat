use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

use axum::extract::Path;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

type UsersWaiting = Arc<Mutex<Vec<(UserId, oneshot::Sender<UserId>)>>>;
type UserId = Uuid;

static STATE: Lazy<Arc<Mutex<State>>> = Lazy::new(|| {
    let s = State::new();
    Arc::new(Mutex::new(s))
});

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

#[derive(Serialize)]
struct PairResponse {
    peer_id: String,
}

struct Connection {
    aws: WebSocket,
    bws: WebSocket,
}

impl Connection {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(message) = self.aws.next() => {
                    match message {
                        Ok(msg) => {
                            if self.bws.send(msg).await.is_err() {
                                eprintln!("Failed to send message to bws");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from aws: {:?}", e);
                            break;
                        }
                    }
                }
                Some(message) = self.bws.next() => {
                    match message {
                        Ok(msg) => {
                            if self.aws.send(msg).await.is_err() {
                                eprintln!("Failed to send message to aws");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from bws: {:?}", e);
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
    users_waiting: UsersWaiting,
    // Map of all pairs that have yet to establish a connection.
    pairs: Pairs,
    /// There's a brief period where one user has established a connection
    /// but his peer have not.
    cons: HashMap<UserId, WebSocket>,
}

impl State {
    fn new() -> Self {
        let s = Self::default();
        s.start_pairing();
        s
    }

    /// Queues a user for pairing. Await the oneshot receiver and
    /// you will receive the peer ID when pairing has completed.
    fn queue(&self, id: UserId) -> oneshot::Receiver<UserId> {
        let (tx, rx) = oneshot::channel();
        self.users_waiting.lock().unwrap().push((id, tx));
        rx
    }

    fn start_pairing(&self) {
        let users = Arc::clone(&self.users_waiting);

        std::thread::spawn(move || loop {
            {
                let mut lock = users.lock().unwrap();
                if lock.len() > 1 {
                    let (first_id, first_sender) = lock.pop().unwrap();
                    let (sec_id, sec_sender) = lock.pop().unwrap();
                    first_sender.send(sec_id).unwrap();
                    sec_sender.send(first_id).unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        });
    }
}

pub async fn pair_handler() -> impl IntoResponse {
    let user_id = Uuid::new_v4();
    println!("Generated UUID for new user: {}", user_id);

    let peer_id = {
        let state = STATE.lock().unwrap();
        state.queue(user_id)
    };

    let peer_id = peer_id.await.unwrap();
    {
        let mut state = STATE.lock().unwrap();
        state.pairs.insert(user_id, peer_id);
    }
    let peer_id = peer_id.to_string();
    Json(PairResponse { peer_id })
}

pub async fn connect_handler(ws: WebSocketUpgrade, Path(id): Path<String>) -> impl IntoResponse {
    let state = STATE.clone();

    let id: UserId = id.parse().unwrap();
    let peer = state.lock().unwrap().pairs.get(id);

    ws.on_upgrade(move |socket| {
        let state = state.clone();
        async move {
            let mut state = state.lock().unwrap();
            if let Some(peer_socket) = state.cons.remove(&peer) {
                let mut con = Connection {
                    aws: peer_socket,
                    bws: socket,
                };

                state.pairs.remove(id);

                tokio::spawn(async move {
                    con.run().await;
                });
            } else {
                state.cons.insert(id, socket);
            }
        }
    })
}

pub fn run() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/pair", get(pair_handler))
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
