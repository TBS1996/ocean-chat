use crate::common;
use axum::extract::ws::{Message, WebSocket};
use common::Scores;
use common::SocketMessage;
use futures_util::StreamExt;
use std::time::SystemTime;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use futures_util::SinkExt;
//use futures_util::StreamExt;

pub struct SocketStuff {
    inner: WebSocket,
}

impl SocketStuff {
    pub fn new(
        socket: WebSocket,
        mut x_receiver: Receiver<SocketMessage>,
    ) -> Receiver<SocketMessage> {
        let (sender, receiver) = channel(32);

        tokio::spawn(async move {
            let (mut tx, mut rx) = socket.split();

            loop {
                tokio::select! {
                    Some(socketmessage) = x_receiver.recv() => {
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

        receiver
    }
}

pub struct User {
    pub scores: Scores,
    pub id: String,
    pub con_time: SystemTime,
    pub receiver: Receiver<SocketMessage>,
    pub sender: Sender<SocketMessage>,
}

impl User {
    pub fn is_legit(&mut self) -> bool {
        while let Ok(msg) = self.receiver.try_recv() {
            if matches!(msg, SocketMessage::ConnectionClosed) {
                return false;
            }
        }

        true
    }
}
