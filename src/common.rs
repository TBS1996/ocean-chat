use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::num::ParseFloatError;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::load()));

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub backend_ip: String,
    pub pair_interval_millis: u64,
}

impl Config {
    pub fn load() -> Self {
        let path = PathBuf::from("config.toml");

        if !path.exists() {
            let s: String = toml::to_string(&Self::default()).unwrap();
            fs::write(&path, s.as_bytes()).unwrap();
        }

        let s: String = fs::read_to_string(&path).unwrap();

        toml::from_str(&s).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            backend_ip: "127.0.0.1".to_string(),
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
