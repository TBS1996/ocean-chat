use crate::common;

use axum::extract::ws::{Message, WebSocket};
use common::CONFIG;
use common::{Scores, SocketMessage};
use futures_util::StreamExt;
use std::time::SystemTime;

use futures_util::SinkExt;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{sleep, Duration, Instant};

/// Takes care of sending to and receiving from a websocket.
fn handle_socket(
    socket: WebSocket,
    id: String,
) -> (Sender<SocketMessage>, Receiver<SocketMessage>) {
    let (x_sender, mut x_receiver) = channel::<SocketMessage>(32);
    let (sender, receiver) = channel(32);

    let mut closed = false;

    tokio::spawn(async move {
        let (mut tx, mut rx) = socket.split();
        let timeout_duration = Duration::from_secs(CONFIG.timeout_secs);
        let timeout = sleep(timeout_duration);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                Some(socketmessage) = x_receiver.recv() => {
                    let _ = tx.send(socketmessage.into_message()).await;
                },

                Some(Ok(msg)) = rx.next() => {
                    match msg {
                        Message::Close(_) => {
                            tracing::info!("{}: client closed connection", &id);
                            closed = true;

                           // let _ = sender.send(SocketMessage::ConnectionClosed).await;
                           // break;
                        },
                        Message::Binary(bytes) => {
                            if closed {
                                // This shouldn't be possible.
                                tracing::error!("{}: received message after closed client", &id);
                            }

                            match serde_json::from_slice(&bytes) {
                                Ok(SocketMessage::Ping) => {
                                    timeout.as_mut().reset(Instant::now() + timeout_duration);
                                },
                                Ok(socket_message) => {
                                    let _ = sender.send(socket_message).await;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },

                _ = &mut timeout => {
                    tracing::info!("{}: Timeout occurred, closing connection", &id);
                    let _ = sender.send(SocketMessage::ConnectionClosed).await;
                    break;
                }
            }
        }
    });

    (x_sender, receiver)
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub scores: Scores,
    pub con_time: SystemTime,
    pub sender: Sender<SocketMessage>,
    pub receiver: Receiver<SocketMessage>,
}

impl User {
    pub fn new(scores: Scores, id: String, socket: WebSocket) -> Self {
        tracing::info!("user queued ");
        let con_time = SystemTime::now();

        let (sender, receiver) = handle_socket(socket, id.clone());

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

    pub async fn receive(&mut self) -> Option<SocketMessage> {
        self.receiver.recv().await
    }

    pub fn is_closed(&mut self) -> bool {
        while let Ok(msg) = self.receiver.try_recv() {
            if matches!(msg, SocketMessage::ConnectionClosed) {
                tracing::info!("user closed: {}", &self.id);
                return true;
            }
        }

        false
    }
}
