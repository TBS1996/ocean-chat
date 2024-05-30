use crate::common::SocketMessage;
use axum::{extract::ws::Message, extract::ws::WebSocket};
use futures_util::SinkExt;
use futures_util::StreamExt;

/// Holds the client-server connections between two peers.
pub struct Connection {
    left: WebSocket,
    right: WebSocket,
}

impl Connection {
    pub fn new(left: WebSocket, right: WebSocket) -> Self {
        Self { left, right }
    }

    /// Handles sending messages from one peer to another.
    pub async fn run(self) {
        tracing::info!("communication starting between a pair");
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
                            if left_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                tracing::error!("Failed to send message to left");
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
                            if right_tx.send(SocketMessage::user_msg(msg)).await.is_err() {
                                tracing::error!("Failed to send message to right");
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
