#![allow(dead_code)]

#[cfg(not(feature = "server"))]
mod client;

mod common;

#[cfg(not(feature = "server"))]
pub use client::run_app;