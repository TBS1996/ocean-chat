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

#[derive(Debug)]
pub enum SLOAN {
    Rcoai,
    Rcoan,
    Rloan,
    Rloai,
    Rluai,
    Rluan,
    Rcuan,
    Rcuai,
    Rcoei,
    Rcoen,
    Rloen,
    Rloei,
    Rluei,
    Rluen,
    Rcuen,
    Rcuei,
    Scoai,
    Scoan,
    Sloan,
    Sloai,
    Sluai,
    Sluan,
    Scuan,
    Scuai,
    Scoei,
    Scoen,
    Sloen,
    Sloei,
    Sluei,
    Sluen,
    Scuen,
    Scuei,
}

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
impl Display for SLOAN {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SLOAN {
    pub fn from_scores(scores: Scores) -> Self {
        let s = scores.e > 50.0;
        let l = scores.n > 50.0;
        let o = scores.c > 50.0;
        let a = scores.a > 50.0;
        let i = scores.o > 50.0;

        match (s, l, o, a, i) {
            (false, false, false, false, false) => Self::Rcuan,
            (false, false, false, false, true) => Self::Rcuai,
            (false, false, false, true, false) => Self::Rcoan,
            (false, false, false, true, true) => Self::Rcoai,
            (false, false, true, false, false) => Self::Rluan,
            (false, false, true, false, true) => Self::Rluai,
            (false, false, true, true, false) => Self::Rloan,
            (false, false, true, true, true) => Self::Rloai,
            (false, true, false, false, false) => Self::Rcuen,
            (false, true, false, false, true) => Self::Rcuei,
            (false, true, false, true, false) => Self::Rcoen,
            (false, true, false, true, true) => Self::Rcoei,
            (false, true, true, false, false) => Self::Rluen,
            (false, true, true, false, true) => Self::Rluei,
            (false, true, true, true, false) => Self::Rloen,
            (false, true, true, true, true) => Self::Rloei,
            (true, false, false, false, false) => Self::Scuan,
            (true, false, false, false, true) => Self::Scuai,
            (true, false, false, true, false) => Self::Scoan,
            (true, false, false, true, true) => Self::Scoai,
            (true, false, true, false, false) => Self::Sluan,
            (true, false, true, false, true) => Self::Sluai,
            (true, false, true, true, false) => Self::Sloan,
            (true, false, true, true, true) => Self::Sloai,
            (true, true, false, false, false) => Self::Scuen,
            (true, true, false, false, true) => Self::Scuei,
            (true, true, false, true, false) => Self::Scoen,
            (true, true, false, true, true) => Self::Scoei,
            (true, true, true, false, false) => Self::Sluen,
            (true, true, true, false, true) => Self::Sluei,
            (true, true, true, true, false) => Self::Sloen,
            (true, true, true, true, true) => Self::Sloei,
        }
    }
}

#[cfg(feature = "server")]
use axum::extract::ws::Message;

/// The type that gets sent from server to client through socket.
#[derive(Serialize, Deserialize)]
pub enum SocketMessage {
    User(String),
    Info(String),
    PeerScores(Scores),
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
}
