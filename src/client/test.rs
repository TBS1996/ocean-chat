#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::save_scores;
use client::Navbar;
use client::Route;
use client::State;
use common::Answer;
use common::Question;
use common::ScoreTally;
use common::DISTS;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

use super::*;

/// using statics everywhere because im too dumb to understand dioxus properly
static QUESTIONS: Lazy<Arc<Mutex<Vec<Question>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Question::iter().collect())));

static TALLY: Lazy<Arc<Mutex<ScoreTally>>> =
    Lazy::new(|| Arc::new(Mutex::new(ScoreTally::default())));

#[component]
pub fn Test() -> Element {
    let state = use_context::<State>();
    let mut curr_question = use_signal(|| QUESTIONS.lock().unwrap().last().copied().unwrap());
    let navigator = use_navigator();
    let mut percentage_done = use_signal(|| 0.);

    let show_navbar = state.scores().is_some();

    rsx! {
                style { { include_str!("manual.css") } },
                if show_navbar { Navbar{active_chat: false} } else {  { top_bar() } }
                div {
                    display: "flex",
                    justify_content: "center",
                    margin: "20px",
                    flex_direction: "column",
                    align_items: "center",

                    div {
                        display: "flex",
                        justify_content: "center",
                        font_size: "1.5em",
                        padding_bottom: "30px",

                        "{curr_question}"
                    }

                    div { class: "buttons",
                        for (answer, state) in Answer::ALL.iter().zip(std::iter::repeat(state.clone())) {
                            button {
                                class: "mybutton",
                                prevent_default: "onclick",
                                onclick: move |_| {
                                    let question = QUESTIONS.lock().unwrap().pop().unwrap();
                                    {
                                        TALLY.lock().unwrap().add_answer(question, *answer);
                                    }

                                    let questions_left = QUESTIONS.lock().unwrap().len();
                                    *percentage_done.write() = ((50 - questions_left) as f32 / 50.) * 100.;

                                    match QUESTIONS.lock().unwrap().last().copied() {
                                        Some(next_question) => {
                                            *curr_question.write() = next_question;
                                        },
                                        None => {
                                            let tally = {
                                                *TALLY.lock().unwrap()
                                            };

                                            let scores = DISTS.convert(tally);
                                            save_scores(scores);
                                            state.set_scores(scores);
                                            navigator.replace(Route::Personality{});
                                        },
                                    }
                                },
                                "{answer}"
                            }
                        }
                    }
                }

                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",
                        flex_direction: "column",
                        p {"{percentage_done}%"}
                        div {
                            display: "flex",
                            justify_content: "left",
                            background_color: "#f1f1f1",
                            overflow: "hidden",
                            height: "30px",
                            width: "500px",
                            div {
                                display: "flex",
                                align_items: "left",
                                justify_content: "center",
                                height: "100%",
                                color: "white",
                                border_radius: "25px 0 0 25px",
                                transition: "width 0.5s",
                                width: "{percentage_done}%",
                                background_color: "red",
                            }
                        }
                        { manual_msg() }
            }
    }
}
