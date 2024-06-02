#![allow(dead_code)]

use common::Distributions;
use common::ScoreTally;
use common::Scores;
use std::io::Write;

#[cfg(feature = "server")]
mod server;

mod common;

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    generate_files();
    scores();
    return;
    server::run().await;
}

#[cfg(not(feature = "server"))]
fn main() {
    ocean_chat::run_app();
}

fn scores() {
    let my_score = Scores {
        a: 27.,
        o: 95.,
        c: 3.,
        e: 95.,
        n: 5.,
    };

    println!("analyzing scores: {:?}", my_score);
    let weird = weirdness_percent(my_score);
    println!(
        "the provided scores are weirder than: {:.2}% of the population!",
        weird
    );
}

// Gives your percentage of weirdness.
//
// 0% => You are extraordinarily ordinary
// 100% => you're a weirdo
fn weirdness_percent(arg: Scores) -> f32 {
    let mid = Scores::mid();

    let weirdness = arg.distance(&mid);

    let mut diffs: Vec<f32> = common::SCORES
        .iter()
        .map(|score| mid.distance(score))
        .collect();

    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let position = diffs
        .iter()
        .position(|diff| *diff > weirdness)
        .unwrap_or(diffs.len());

    let ratio = position as f32 / diffs.len() as f32;
    ratio * 100.
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
