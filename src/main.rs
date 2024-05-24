use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, Response},
    routing::get,
    Router,
};
use color_eyre::eyre::Result;
use futures_util::StreamExt;
use rand::Rng;
use std::{
    collections::HashSet,
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};
use tower_http::trace::TraceLayer;

use tracing::{error, info};
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<HashSet<u64>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bigly=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = AppState {
        users: Arc::new(Mutex::new(HashSet::new())),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/chat", get(chat))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let socket_addr = (Ipv4Addr::new(127, 0, 0, 1), 3030);
    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    info!("listening at address {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        "
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset=\"utf-8\">
            <title>Chat App</title>
        </head>
        <body>
            <h1>Chat App</h1>
            <input type=\"text\" id=\"message\" placeholder=\"Type a message...\">
            <button onclick=\"sendMessage()\">Send</button>
            <ul id=\"messages\"></ul>
            <script>
                // Establish a WebSocket connection to the server
                const ws = new WebSocket('ws://' + window.location.host + '/chat');

                // Handle incoming messages
                ws.onmessage = (event) => {
                    const messages = document.getElementById('messages');
                    const message = document.createElement('li');
                    message.textContent = event.data;
                    messages.appendChild(message);
                };

                // Send a message when the button is clicked
                function sendMessage() {
                    const input = document.getElementById('message');
                    ws.send(input.value);
                    input.value = '';
                }
            </script>
        </body>
        </html>
        ",
    )
}

async fn chat(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let user_id = rand::thread_rng().gen::<u64>();
    ws.on_upgrade(move |socket| handle_chat_websocket(socket, state, user_id))
}

async fn handle_chat_websocket(ws: WebSocket, state: AppState, user_id: u64) {
    // Split the WebSocket connection into a sender (tx) and receiver (rx)
    let (mut _sender, mut receiver) = ws.split();

    // Add the new user to the set of connected users
    state.users.lock().unwrap().insert(user_id);
    info!("successful connection from uid={user_id}");

    // Loop to receive messages from the WebSocket connection
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                if let Message::Text(msg) = msg {
                    broadcast_message(user_id, msg, &state).await;
                }
            }
            Err(e) => {
                error!(error = e.to_string(), "websocket error from uid={user_id}");
                break;
            }
        }
    }

    // Remove the user from the set of connected users when they disconnect
    state.users.lock().unwrap().insert(user_id);
    info!("successful disconnection from uid={user_id}");
}

// Function to broadcast a message to all connected users except the sender
async fn broadcast_message(user_id: u64, _msg: String, state: &AppState) {
    info!("broadcasting message from uid={user_id}");
    for &uid in state.users.lock().unwrap().iter() {
        // Skip the sender of the message
        if uid != user_id {
            // Placeholder for the actual sending logic
            // Here you would actually send the message to other users
        }
    }
}
