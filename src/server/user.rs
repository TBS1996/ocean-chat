use crate::common;
use axum::extract::ws::{Message, WebSocket};
use common::Scores;
use common::SocketMessage;
use std::time::SystemTime;

pub struct User {
    pub scores: Scores,
    pub socket: WebSocket,
    pub id: String,
    pub con_time: SystemTime,
}

impl User {
    pub async fn ping(&mut self) -> bool {
        let ping_timeout = tokio::time::Duration::from_millis(2000);
        if self.socket.send(SocketMessage::ping()).await.is_err() {
            return false;
        }

        while let Ok(Some(Ok(Message::Binary(msg)))) =
            tokio::time::timeout(ping_timeout, self.socket.recv()).await
        {
            if let Ok(SocketMessage::Pong) = serde_json::from_slice(&msg) {
                return true;
            }
        }

        false
    }

    pub async fn drain_socket(&mut self) {
        let drain_timeout = tokio::time::Duration::from_millis(100);
        while let Ok(Some(_)) = tokio::time::timeout(drain_timeout, self.socket.recv()).await {}
    }
}
