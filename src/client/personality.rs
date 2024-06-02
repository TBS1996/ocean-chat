#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::Route;
use client::Sidebar;
use client::State;
use common::Scores;
use common::Trait;
use dioxus::prelude::*;

#[component]
pub fn Personality() -> Element {
    let state = use_context::<State>();
    let scores = state.scores().unwrap();

    rsx! {
        div {
        class: "layout",
        Sidebar{}
        div {
            style { { include_str!("personality.css") } },

            div { class: "container",
                h1 { "Your big five scores!" }
                PercentileBar { tr: Trait::Open, score: scores.o }
                PercentileBar { tr: Trait::Con, score: scores.c }
                PercentileBar { tr: Trait::Extro, score: scores.e }
                PercentileBar { tr: Trait::Agree, score: scores.a }
                PercentileBar { tr: Trait::Neurotic, score: scores.n }
            }
        }
    }
    }
}

#[component]
fn PercentileBar(tr: Trait, score: f32) -> Element {
    let bar_style = format!("width: {}%; background-color: {}", score, tr.color());

    rsx! {
        div { class: "bar-row",
            div {class: "label", "{tr}"},
            div { class: "bar-container",
                div { class: "bar", style: "{bar_style}",
                    "{score}%"
                }
            }
        }
    }
}
