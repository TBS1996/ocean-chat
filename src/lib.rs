#![allow(dead_code)]

#[cfg(not(feature = "server"))]
pub mod client;

mod common;

#[cfg(not(feature = "server"))]
pub use client::run_app;
