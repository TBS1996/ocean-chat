#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::log_to_console;
use common::Trait;

use client::save_scores;
use client::test_msg;
use client::top_bar;
use client::utils::footer;
use client::Navbar;
use client::Route;
use client::State;
use common::Scores;
use dioxus::prelude::*;
use std::ops::Deref;

#[component]
pub fn Manual() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();
    let score = state.scores();
    let show_sidebar = state.scores().is_some();

    let traits: Vec<Trait> = vec![
        Trait::Open,
        Trait::Con,
        Trait::Extro,
        Trait::Agree,
        Trait::Neurotic,
    ];

    let default_vals: Vec<String> = traits
        .iter()
        .map(|tr| {
            score
                .map(|score| format!("{:.1}", score.trait_val(*tr)))
                .unwrap_or_default()
        })
        .collect();

    let trait_vals = traits.iter().zip(default_vals.iter());

    rsx! {
        if show_sidebar {Navbar {active_chat: false}} else { { top_bar() } },
        div {
            padding_top: "20px",
            display: "flex",
            justify_content: "center",
            flex_direction: "column",
            align_content: "center",
            form {
                onsubmit:  move |event| {
                    log_to_console("clicked submit");
                    match Scores::try_from(event.data().deref()) {
                        Ok(scores) => {
                            state.set_scores(scores);
                            save_scores(scores);
                            navigator.replace(Route::Personality{});
                        }
                        Err(_) => {
                            navigator.replace(Route::Invalid {});
                        }
                    }
                },
                div {
                    display: "flex",
                    justify_content: "center",
                    flex_direction: "row",
                    align_content: "center",
                    div {
                        display: "flex",
                        justify_content: "center",
                        flex_direction: "column",
                        align_content: "center",
                        for (t, val) in trait_vals{
                            div {
                                display: "flex",
                                justify_content: "space-between",
                                label { r#for: "{t}", "{t}: " }
                                div {
                                    input { id: "{t}", name: "{t}", value: "{val}", r#type: "number", step: "any", min: "0", max: "100", required: true}
                                    " %"
                                }
                            }
                        }
                    }
                }
                div {
                    display: "flex",
                    flex_direction: "row",
                    justify_content: "center",
                    padding_top: "10px",
                    button {
                        width: "250px",
                        class: "mybutton",
                        r#type: "submit",
                        h2 { "Save" }
                    }
                }
                div {
                    display: "flex",
                    justify_content: "center",
                    { test_msg() }
                }
            }
      //      { footer() }
        }

    }
}
