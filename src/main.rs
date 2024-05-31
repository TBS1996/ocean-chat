#![allow(dead_code)]

#[cfg(feature = "server")]
mod server;

mod common;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    server::run().await;
}

#[cfg(not(feature = "server"))]
fn main() {
    ocean_chat::run_app();
}
