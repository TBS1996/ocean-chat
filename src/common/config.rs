use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::load()));

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_local")]
    pub local: bool,
    #[serde(default = "default_pair_interval_millis")]
    pub pair_interval_millis: u64,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
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
        let config_str = include_str!("../../config.toml");
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
            "wss://95.179.226.104"
        }
    }
}

fn default_pair_interval_millis() -> u64 {
    1000
}

fn default_local() -> bool {
    false
}

fn default_timeout() -> u64 {
    120
}

impl Default for Config {
    fn default() -> Self {
        Config {
            local: default_local(),
            pair_interval_millis: default_pair_interval_millis(),
            timeout_secs: default_timeout(),
        }
    }
}
