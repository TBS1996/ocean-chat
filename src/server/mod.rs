use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::Extension,
    extract::Path,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::common::Scores;
use crate::common::CONFIG;
use crate::server::connection::Connection;
use crate::server::waiting_users::WaitingUser;
use crate::server::waiting_users::WaitingUsers;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod connection;
mod waiting_users;

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
    async fn queue(&self, score: Scores, socket: WebSocket) {
        eprintln!("user queued ");
        let user = WaitingUser { score, socket };
        self.waiting_users.queue(user).await;
    }

    async fn start_pairing(&self) {
        eprintln!("pairing started");
        let users = self.waiting_users.clone();
        tokio::spawn(async move {
            loop {
                {
                    while let Some((left, right)) = users.pop_pair().await {
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
            state.queue(scores, socket).await;
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

    let addr = "127.0.0.1:3000".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
