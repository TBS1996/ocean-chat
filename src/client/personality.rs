#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::Route;
use client::Sidebar;
use client::Splash;
use client::State;
use common::Scores;
use common::Trait;
use dioxus::prelude::*;

#[component]
pub fn Personality() -> Element {
    let state = use_context::<State>();
    let Some(scores) = state.scores() else {
        return Splash();
    };

    let weirdness = scores.weirdness_percent() as u32;

    rsx! {
        div {
        class: "layout",
        Sidebar{}
        div {
            style { { include_str!("personality.css") } },

            div { class: "container",
                h1 { "Your big five scores!" }
                PercentileBar { tr: Trait::Open, score: scores.o as u32 }
                PercentileBar { tr: Trait::Con, score: scores.c as u32}
                PercentileBar { tr: Trait::Extro, score: scores.e as u32}
                PercentileBar { tr: Trait::Agree, score: scores.a as u32}
                PercentileBar { tr: Trait::Neurotic, score: scores.n as u32}
            }
        }
        div {
            Link {
                to: Route::Manual {},
                "Edit values"
            }
        }
        div {
            Link {
                to: Route::Test {},
                "Take the test again"
            }
        }
        div {
            "you are weirder than {weirdness}% of people"
        }
    }
    }
}

#[component]
fn PercentileBar(tr: Trait, score: u32) -> Element {
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
