#![allow(dead_code)]

#[cfg(not(feature = "server"))]
mod frontend;

mod common;

#[cfg(not(feature = "server"))]
pub use frontend::run_app;
