use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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
