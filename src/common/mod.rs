#![allow(unused_imports)]

use serde::{Deserialize, Serialize};

pub mod config;
pub mod distribution;
pub mod questions;
pub mod scores;
pub mod sloan;

pub use config::*;
pub use distribution::*;
pub use questions::*;
pub use scores::*;
pub use sloan::*;

#[cfg(feature = "server")]
use axum::extract::ws::Message;

/// The type that gets sent from server to client through socket.
#[derive(Debug, Serialize, Deserialize)]
pub enum SocketMessage {
    User(String),
    Info(String),
    PeerScores(Scores),
    ConnectionClosed,
    Ping,
    Pong,
}

/// Messages being sent from the server to the client.
#[cfg(feature = "server")]
impl SocketMessage {
    pub fn user_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::User(msg)).unwrap();
        Message::Text(s)
    }

    pub fn info_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::Info(msg)).unwrap();
        Message::Text(s)
    }

    pub fn peer_scores(scores: Scores) -> Message {
        let s = serde_json::to_string(&Self::PeerScores(scores)).unwrap();
        Message::Text(s)
    }

    pub fn into_message(self) -> Message {
        let s = serde_json::to_string(&self).unwrap();
        Message::Text(s)
    }

    pub fn close_connection() -> Message {
        let s = serde_json::to_string(&Self::ConnectionClosed).unwrap();
        Message::Text(s)
    }

    pub fn ping() -> Message {
        let s = serde_json::to_string(&Self::Ping).unwrap();
        Message::Text(s)
    }
}

/// Messages being sent from the client to the server.
#[cfg(not(feature = "server"))]
impl SocketMessage {
    pub fn user_msg(msg: String) -> Vec<u8> {
        let mut writer: Vec<u8> = vec![];
        let val = Self::User(msg);
        serde_json::to_writer(&mut writer, &val).unwrap();
        writer
    }

    pub fn ping() -> Vec<u8> {
        let mut writer: Vec<u8> = vec![];
        let val = Self::Ping;
        serde_json::to_writer(&mut writer, &val).unwrap();
        writer
    }

    pub fn pong() -> Vec<u8> {
        let mut writer: Vec<u8> = vec![];
        let val = Self::Pong;
        serde_json::to_writer(&mut writer, &val).unwrap();
        writer
    }
}

impl SocketMessage {
    pub fn is_closed(&self) -> bool {
        matches!(&self, SocketMessage::ConnectionClosed)
    }
}
