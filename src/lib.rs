#[cfg(not(feature = "server"))]
mod frontend;

#[cfg(not(feature = "server"))]
pub use frontend::run_app;
