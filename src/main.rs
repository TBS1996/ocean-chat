#![allow(dead_code)]

use common::Distributions;
use common::ScoreTally;
use common::Scores;
use common::CONFIG;
use std::io::Write;

#[cfg(feature = "server")]
mod server;

mod common;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    enforce_release();
    generate_files();
    server::run().await;
}

fn enforce_release() {
    if cfg!(debug_assertions) && !CONFIG.local {
        eprintln!("Server should be run in release mode while in production:");
        eprintln!("`cargo run --features server --release`");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    ocean_chat::run_app();
}

fn generate_files() {
    let tallies = ScoreTally::load();
    let dist = Distributions::from_tallies(&tallies);
    dist.write_to_disk();
    let scores: Vec<Scores> = tallies
        .into_iter()
        .map(|tally| dist.convert(tally))
        .collect();
    let mut output = String::new();

    for score in scores {
        let s = format!("{}\n", score);
        output.push_str(&s);
    }

    let p = std::path::PathBuf::from("files/scores");
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(&output.as_bytes()).unwrap();
}
