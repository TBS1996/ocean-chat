#![allow(non_snake_case)]

use crate::common;

use common::Scores;

use crate::client::Route;

use dioxus::prelude::*;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[component]
pub fn Invalid() -> Element {
    rsx! {
        div {
            p {
                "You have to either take the personality test, or manually submit a valid set of trait scores!"
            }
            div {
                Link {
                    to: Route::Home {},
                    "Back to main page"
                }
            }
        }
    }
}

pub fn save_id(id: Uuid) {
    let script = format!("localStorage.setItem('user_id', '{}');", id);
    eval(&script);
    log_to_console("storing user_id in local storage");
}

pub fn save_scores(scores: Scores) {
    let script = format!("localStorage.setItem('scores', '{}');", scores);
    eval(&script);
    log_to_console("storing scores in local storage");
}

// Call this function to log a message
pub fn log_to_console(message: impl std::fmt::Debug) {
    let message = format!("{:?}", message);
    console::log_1(&JsValue::from_str(&message));
}

#[component]
pub fn Navbar(active_chat: bool) -> Element {
    rsx! {
        nav {
            ul {
               img {
                   src: "logo.png",
                   alt: "Oceanchat Logo",
                   width: "80px",
                   height: "80px",
                   margin_right: "20px",
               }
                li {
                    Link {
                        to: Route::Chat {},
                        "Chat",
                        class: if active_chat { "active" } else { "" }
                    }
                }
                li {
                    Link { to: Route::Personality {},
                    "My personality",
                    class: if !active_chat { "active" } else { "" }

                    }
                }
            }
        }
    }
}

fn default_scores() -> Scores {
    static COOKIE: Lazy<Option<Scores>> = Lazy::new(|| {
        let scores = block_on(fetch_scores_storage());
        scores
    });

    COOKIE.unwrap_or_else(Scores::mid)
}

pub async fn fetch_id_storage() -> Option<Uuid> {
    let eval = eval(
        r#"
        let id = localStorage.getItem('user_id');
        if (id) {
            dioxus.send(id);
        } else {
            dioxus.send(null);
        }
        "#,
    )
    .recv()
    .await;

    log_to_console(&eval);

    let mut id = eval.ok()?.to_string();
    id.remove(0);
    id.pop();
    let uuid = Uuid::from_str(&id);
    log_to_console(&uuid);
    uuid.ok()
}

pub async fn fetch_scores_storage() -> Option<Scores> {
    let mut eval = eval(
        r#"
        let scores = localStorage.getItem('scores');
        if (scores) {
            dioxus.send(scores);
        } else {
            dioxus.send(null);
        }
        "#,
    );

    let scores = eval.recv().await.unwrap().to_string();
    log_to_console(&scores);
    Scores::from_str(&scores).ok()
}

pub fn test_msg() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            font_size: "0.8em",
            align_items: "center",
            color: "#666",
            div {
                "Unsure? Take the "
                Link {
                    to: Route::Test {},
                    "test"
                }
                "."
            }
        }
    }
}

pub fn manual_msg() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            font_size: "0.8em",
            align_items: "center",
            color: "#666",
            div {
                "Already know your score? Enter them "
                Link {
                    to: Route::Manual {},
                    " manually."
                }
            }
        }
    }
}

pub fn score_cmp(mine: Scores, peer: Scores) -> Element {
    let o_diff = (mine.o - peer.o) as i32;
    let c_diff = (mine.c - peer.c) as i32;
    let e_diff = (mine.e - peer.e) as i32;
    let a_diff = (mine.a - peer.a) as i32;
    let n_diff = (mine.n - peer.n) as i32;

    let ostr = format!(
        "{}% {} openness. {}->{}",
        o_diff.abs(),
        if o_diff < 0 { "higher" } else { "lower" },
        mine.o as u32,
        peer.o as u32
    );
    let cstr = format!(
        "{}% {} conscientiousness. {}->{}",
        c_diff.abs(),
        if c_diff < 0 { "higher" } else { "lower" },
        mine.c,
        peer.c
    );
    let estr = format!(
        "{}% {} extroversion. {}->{}",
        e_diff.abs(),
        if e_diff < 0 { "higher" } else { "lower" },
        mine.e,
        peer.e
    );
    let astr = format!(
        "{}% {} agreeableness. {}->{}",
        a_diff.abs(),
        if a_diff < 0 { "higher" } else { "lower" },
        mine.a,
        peer.a
    );
    let nstr = format!(
        "{}% {} neuroticism. {}->{}",
        n_diff.abs(),
        if n_diff < 0 { "higher" } else { "lower" },
        mine.n,
        peer.n
    );

    let font_size = "1em";

    rsx! {
        div {
            p {font_size: "{font_size}", "{ostr}"}
            p {font_size: "{font_size}", "{cstr}"}
            p {font_size: "{font_size}", "{estr}"}
            p {font_size: "{font_size}", "{astr}"}
            p {font_size: "{font_size}", "{nstr}"}
        }
    }
}
