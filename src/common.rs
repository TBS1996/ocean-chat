use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::load()));

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub local: bool,
    pub pair_interval_millis: u64,
}

impl Config {
    /// Loads the config file.
    ///
    /// Behaviour is different between backend and frontend.
    /// In the backend we load the config from file, creating new if it doesn't exist.
    ///
    /// The frontend however, needs to package the config into the binary since it doesn't
    /// have access to the backend's file system. We use include_str which will make the program
    /// fail to compile if we haven't configured a config yet. This means in practice that if you
    /// didn't manually add a config file, you need to run the backend at least once before
    /// you build the frontend.
    pub fn load() -> Self {
        #[cfg(not(feature = "server"))]
        let config_str = include_str!("../config.toml");
        #[cfg(feature = "server")]
        let config_str = {
            let config_path = std::path::PathBuf::from("config.toml");
            if !config_path.exists() {
                let s: String = toml::to_string(&Self::default()).unwrap();
                std::fs::write(&config_path, s.as_bytes()).unwrap();
            }
            std::fs::read_to_string(&config_path).unwrap()
        };

        toml::from_str(&config_str).unwrap()
    }

    pub fn server_address(&self) -> &'static str {
        if self.local {
            "ws://127.0.0.1:3000"
        } else {
            "wss://oceanchat.app"
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            local: true,
            pair_interval_millis: 1000,
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

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Scores {
    pub o: f32,
    pub c: f32,
    pub e: f32,
    pub a: f32,
    pub n: f32,
}

impl Scores {
    /// Calculates euclidean distance between two scores.
    #[allow(dead_code)]
    pub fn distance(&self, other: &Self) -> f32 {
        let open = self.o - other.o;
        let con = self.c - other.c;
        let extro = self.e - other.e;
        let agreeable = self.a - other.a;
        let neurotic = self.n - other.n;

        let diff_sum =
            open.powi(2) + con.powi(2) + extro.powi(2) + agreeable.powi(2) + neurotic.powi(2);

        diff_sum.sqrt()
    }
}

impl Display for Scores {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.o, self.c, self.e, self.a, self.n)
    }
}

impl FromStr for Scores {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(',').collect();

        let o = values[0].parse()?;
        let c = values[1].parse()?;
        let e = values[2].parse()?;
        let a = values[3].parse()?;
        let n = values[4].parse()?;

        Ok(Self { o, c, e, a, n })
    }
}
