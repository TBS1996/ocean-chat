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
use common::Scores;
use common::DISTS;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

#[component]
pub fn Manual() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();
    let score = state.scores().unwrap_or_default();
    let show_sidebar = state.scores().is_some();

    rsx! {
        style { { include_str!("../styles.css") } }
        main {
            class: "layout",
            if show_sidebar {Sidebar {}} else {{}},
            div {
                h1 {"Edit scores"}
                br {}
                form {
                    onsubmit:  move |event| {
                        match Scores::try_from(event.data().deref()) {
                            Ok(scores) => {
                                state.set_scores(scores);
                                save_scores(scores);
                                navigator.replace(Route::Chat{});
                            }
                            Err(_) => {
                                navigator.replace(Route::Invalid {});
                            }
                        }
                    },

                    div {
                        class: "spread-around",
                        label { r#for: "o", "Openness: " }
                        input { id: "o", name: "o", value: "{score.o as u32}", r#type: "number", step: "any", min: "0", max: "100" }
                    }

                    div {
                        class: "spread-around",
                        label { r#for: "c", "Conscientiousness: " }
                        input { id: "c", name: "c", value: "{score.c as u32}", r#type: "number", step: "any", min: "0", max: "100" }
                    }

                    div {
                        class: "spread-around",
                        label { r#for: "e", "Extraversion: " }
                        input { id: "e", name: "e", value: "{score.e as u32}", r#type: "number", step: "any", min: "0", max: "100" }
                    }

                    div {
                        class: "spread-around",
                        label { r#for: "a", "Agreeableness: " }
                        input { id: "a", name: "a", value: "{score.a as u32}", r#type: "number", step: "any", min: "0", max: "100" }
                    }

                    div {
                        class: "spread-around",
                        label { r#for: "n", "Neuroticism: " }
                        input { id: "n", name: "n", value: "{score.n as u32}", r#type: "number", step: "any", min: "0", max: "100" }
                    }

                    br {}

                    button {
                        class: "confirm",
                        r#type: "submit",
                        h2 { "Save" }
                    }
                }

                div {
                    Link {
                        to: Route::Test {},
                        "Or take the test instead"
                    }
                }


            }
        }
    }
}
