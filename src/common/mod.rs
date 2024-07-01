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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub enum UserStatus {
    #[default]
    Disconnected,
    Connected,
    Waiting,
    Idle,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ChangeState {
    Waiting,
    Idle,
}

/// The type that gets sent from server to client through socket.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SocketMessage {
    Status(UserStatus),
    StateChange(ChangeState),
    User(String),
    Info(String),
    PeerScores(Scores),
    ConnectionClosed,
    GetStatus,
    Ping,
    Pong,
}

impl SocketMessage {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = vec![];
        serde_json::to_writer(&mut writer, &self).unwrap();
        writer
    }
}

/// Messages being sent from the server to the client.
#[cfg(feature = "server")]
impl SocketMessage {
    pub fn user_msg(msg: String) -> Message {
        Message::Text(SocketMessage::User(msg).to_string())
    }

    pub fn info_msg(msg: String) -> Message {
        Message::Text(SocketMessage::Info(msg).to_string())
    }

    pub fn state_change(state: ChangeState) -> Message {
        let s = serde_json::to_string(&Self::StateChange(state)).unwrap();
        Message::Text(s)
    }

    pub fn peer_scores(scores: Scores) -> Message {
        Message::Text(SocketMessage::PeerScores(scores).to_string())
    }

    pub fn into_message(self) -> Message {
        Message::Text(self.to_string())
    }

    pub fn close_connection() -> Message {
        Message::Text(SocketMessage::ConnectionClosed.to_string())
    }

    pub fn ping() -> Message {
        Message::Text(SocketMessage::Ping.to_string())
    }
}

/// Messages being sent from the client to the server.
#[cfg(not(feature = "server"))]
impl SocketMessage {
    pub fn user_msg(msg: String) -> Vec<u8> {
        Self::User(msg).to_bytes()
    }

    pub fn get_status() -> Vec<u8> {
        Self::GetStatus.to_bytes()
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
