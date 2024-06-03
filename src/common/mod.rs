#![allow(unused_imports)]

use serde::{Deserialize, Serialize};

pub mod config;
pub mod distribution;
pub mod questions;
pub mod scores;

pub use config::*;
pub use distribution::*;
pub use questions::*;
pub use scores::*;

#[cfg(feature = "server")]
use axum::extract::ws::Message;

/// The type that gets sent from server to client through socket.
#[derive(Serialize, Deserialize)]
pub enum SocketMessage {
    User(String),
    Info(String),
    PeerScores(Scores),
    ConnectionClosed,
    //    Ping,
    //    Pong,
}

impl SocketMessage {
    #[cfg(feature = "server")]
    pub fn user_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::User(msg)).unwrap();
        Message::Text(s)
    }

    #[cfg(feature = "server")]
    pub fn info_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::Info(msg)).unwrap();
        Message::Text(s)
    }

    #[cfg(feature = "server")]
    pub fn peer_scores(scores: Scores) -> Message {
        let s = serde_json::to_string(&Self::PeerScores(scores)).unwrap();
        Message::Text(s)
    }

    #[cfg(feature = "server")]
    pub fn close_connection() -> Message {
        let s = serde_json::to_string(&Self::ConnectionClosed).unwrap();
        Message::Text(s)
    }
}
