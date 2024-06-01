#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::save_scores;
use client::Route;
use client::Sidebar;
use client::State;
use common::Answer;
use common::Question;
use common::ScoreTally;
use common::DISTS;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

/// using statics everywhere because im too dumb to understand dioxus properly
static QUESTIONS: Lazy<Arc<Mutex<Vec<Question>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Question::iter().collect())));

#[component]
pub fn Test() -> Element {
    let state = use_context::<State>();
    let mut tally = use_signal(ScoreTally::default);
    let mut curr_question = use_signal(|| QUESTIONS.lock().unwrap().last().copied().unwrap());
    let navigator = use_navigator();

    rsx! {
        div {
        class: "layout",
        Sidebar {},
        div {
            style { { include_str!("../styles.css") } }
            h1 { "Personality Test" }
            div { class: "input-group",
                "{curr_question}"
            }
            div { class: "buttons",
                for (answer, state) in Answer::ALL.iter().zip(std::iter::repeat(state.clone())) {
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, *answer);
                            match QUESTIONS.lock().unwrap().last().copied() {
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = DISTS.convert(*tally.read());
                                    save_scores(scores);
                                    state.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }
                        },
                        "{answer}"
                    }
                }
            }
        }
    }
    }
}
