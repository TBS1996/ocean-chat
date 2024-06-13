use crate::common;

use axum::extract::ws::{Message, WebSocket};
use common::Scores;
use common::SocketMessage;
use futures_util::StreamExt;
use std::time::SystemTime;

use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use futures_util::SinkExt;

/// Takes care of sending to and receiving from a websocket.
fn handle_socket(socket: WebSocket) -> (Sender<SocketMessage>, Receiver<SocketMessage>) {
    let (x_sender, mut x_receiver) = channel::<SocketMessage>(32);
    let (sender, receiver) = channel(32);

    tokio::spawn(async move {
        let (mut tx, mut rx) = socket.split();

        loop {
            tokio::select! {
                Some(socketmessage) = x_receiver.recv() => {
                    dbg!(&socketmessage);
                    let _ = tx.send(socketmessage.into_message()).await;
                },

                Some(Ok(msg)) = rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            tracing::info!("right closed connection");
                            let _ = sender.send(SocketMessage::ConnectionClosed).await;
                            break;
                        },
                        Message::Binary(bytes) => {
                            match serde_json::from_slice(&bytes) {
                                Ok(socket_message) => {
                                    let _ = sender.send(socket_message).await;

                                    },
                                _ => {},
                                }
                            }
                        _ => {},
                        }
                    }
                else => {}
            }
        }
    });

    (x_sender, receiver)
}

pub struct User {
    pub scores: Scores,
    pub id: String,
    pub con_time: SystemTime,
    pub receiver: Receiver<SocketMessage>,
    pub sender: Sender<SocketMessage>,
}

impl User {
    pub fn new(scores: Scores, id: String, socket: WebSocket) -> Self {
        tracing::info!("user queued ");
        let con_time = SystemTime::now();

        let (sender, receiver) = handle_socket(socket);

        User {
            scores,
            sender,
            receiver,
            id,
            con_time,
        }
    }

    pub async fn send(&mut self, msg: SocketMessage) -> Result<(), SendError<SocketMessage>> {
        self.sender.send(msg).await
    }

    pub fn is_legit(&mut self) -> bool {
        while let Ok(msg) = self.receiver.try_recv() {
            if matches!(msg, SocketMessage::ConnectionClosed) {
                tracing::info!("user closed: {}", &self.id);
                return false;
            }
        }

        true
    }
}
