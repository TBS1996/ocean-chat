#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::Navbar;
use client::Route;
use client::State;
use common::Sloan;
use common::Trait;
use dioxus::prelude::*;

#[component]
pub fn Personality() -> Element {
    let state = use_context::<State>();
    let scores = state.scores().unwrap();
    let sloan = Sloan::from_scores(scores);
    let sloan = format!("{:?}", sloan).to_lowercase();
    let weirdness = format!("{:.2}", scores.weirdness_percent());
    let link = format!("https://similarminds.com/global5/{}.html", sloan);

    rsx! {
        div {
        class: "layout",
        Navbar{active_chat: false}
        div {
            style { { include_str!("personality.css") } },

            div { class: "container",
                h1 { "Your big five scores!" }
                PercentileBar { tr: Trait::Open, score: scores.o as u32 }
                PercentileBar { tr: Trait::Con, score: scores.c as u32}
                PercentileBar { tr: Trait::Extro, score: scores.e as u32}
                PercentileBar { tr: Trait::Agree, score: scores.a as u32}
                PercentileBar { tr: Trait::Neurotic, score: scores.n as u32}
                h2 {"Your type is ", a {
                    href: link,
                    target: "_blank",
                    "{sloan}"
                } },
                h2 {"You are weirder than {weirdness}% of the population!"},
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
