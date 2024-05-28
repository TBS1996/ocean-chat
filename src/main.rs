#[cfg(feature = "server")]
mod server;

mod common;

#[cfg(feature = "server")]
fn main() {
    server::run();
}

#[cfg(not(feature = "server"))]
fn main() {
    ocean_chat::run_app();
}
