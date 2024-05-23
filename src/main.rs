use futures_util::{SinkExt, StreamExt}; // Import Stream and Sink extensions from futures-util
use rand::Rng;
use std::collections::HashSet; // Import HashSet for storing user IDs
use std::sync::{Arc, Mutex}; // Import Arc and Mutex for thread-safe shared state
use warp::ws::{Message, WebSocket}; // Import WebSocket-related types from Warp
use warp::Filter; // Import the Warp web framework // Import random number generator from the rand crate

// Type alias for a thread-safe set of user IDs
type Users = Arc<Mutex<HashSet<usize>>>;

// Function to handle each WebSocket connection
async fn handle_socket(ws: WebSocket, users: Users, user_id: usize) {
    // Split the WebSocket connection into a sender (tx) and receiver (rx)
    let (mut tx, mut rx) = ws.split();

    // Add the new user to the set of connected users
    users.lock().unwrap().insert(user_id);

    // Loop to receive messages from the WebSocket connection
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                // If the message is text or binary, broadcast it to other users
                if msg.is_text() || msg.is_binary() {
                    broadcast_message(user_id, msg, &users).await;
                }
            }
            Err(e) => {
                // If there's an error, log it and break the loop
                eprintln!("websocket error(uid={}): {}", user_id, e);
                break;
            }
        }
    }

    // Remove the user from the set of connected users when they disconnect
    users.lock().unwrap().remove(&user_id);
}

// Function to broadcast a message to all connected users except the sender
async fn broadcast_message(user_id: usize, msg: Message, users: &Users) {
    // Lock the users set to safely iterate over it
    let users = users.lock().unwrap();
    for &uid in users.iter() {
        // Skip the sender of the message
        if uid != user_id {
            // Placeholder for the actual sending logic
            // Here you would actually send the message to other users
        }
    }
}

// Main function to start the server
#[tokio::main] // The main function will use the Tokio runtime
async fn main() {
    // Initialize the set of connected users
    let users = Users::new(Mutex::new(HashSet::new()));

    // Define the WebSocket route at /chat
    let chat_route = warp::path("chat")
        .and(warp::ws()) // Match WebSocket requests
        .and(with_users(users.clone())) // Pass the users set to the handler
        .map(|ws: warp::ws::Ws, users| {
            // Generate a random user ID
            let user_id = rand::thread_rng().gen::<usize>();
            // Upgrade the connection to a WebSocket and handle it
            ws.on_upgrade(move |socket| handle_socket(socket, users, user_id))
        });

    // Define the route to serve the HTML page at /
    let index_route = warp::path::end().map(|| {
        // Serve a simple HTML page with embedded JavaScript
        warp::reply::html(
            "<!DOCTYPE html>
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
                </html>",
        )
    });

    // Combine the routes
    let routes = chat_route.or(index_route);

    // Start the Warp server on 127.0.0.1:3030
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// Helper function to pass the users set to the handler
fn with_users(
    users: Users,
) -> impl Filter<Extract = (Users,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || users.clone())
}

