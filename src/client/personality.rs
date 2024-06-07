#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::markdown_converter;
use client::Navbar;
use client::Route;
use client::State;
use common::Sloan;
use common::Trait;
use dioxus::prelude::*;

use common::Scores;

#[component]
pub fn Personality() -> Element {
    let state = use_context::<State>();
    let scores = state.scores().unwrap();
    let sloan = Sloan::from_scores(scores);
    let summary = sloan.summary();
    let summary = markdown_converter(summary);
    let sloan = format!("{:?}", sloan).to_lowercase();
    let weirdness = scores.weirdness_percent() as u32;
    let link = format!("https://similarminds.com/global5/{}.html", sloan);

    rsx! {
        div {
        Navbar{active_chat: false}
        div {
            style { { include_str!("personality.css") } },

            div {
                width: "50%",
                margin: "auto",
                padding: "20px",
                font_family: "Arial, sans-serif",

                h1 { "Your big five scores!" }
                {  big_five_bars(scores, false) }
                div {
                    display: "flex",
                    flex_direction: "row",
                    justify_content: "left",
                    margin_bottom: "50px",

                    Link {
                        padding_right: "10px",
                        to: Route::Manual {},
                        "Edit values"
                    }
                    Link {
                        to: Route::Test {},
                        "Take the test"
                    }
                }
                h2 {"Your type is ", a {
                    href: link,
                    target: "_blank",
                    "{sloan}"
                } },
                h2 {"You are weirder than {weirdness}% of the population!"},

                div {
                    padding_top: "50px",
                    { summary }

                }
            }
        }
    }
    }
}

#[component]
fn PercentileBar(tr: Trait, score: u32, label_top: bool) -> Element {
    rsx! {
        div {
            display: "flex",
            justify_content: "space-between",
            align_items: "center",
            width: "100%",
            margin: "10px 0",
            flex_direction: if label_top {"column"} else {"row"},

            div {class: "label", "{tr}"},
            { PercentileBarRaw(tr.color(), score) }
        }
    }
}

pub fn PercentileBarRaw(color: &str, score: u32) -> Element {
    rsx! {
        div {
            display: "flex",
            justify_content: "left",
            background_color: "#f1f1f1",
            border_radius: "25px",
            overflow: "hidden",
            height: "30px",
            width: "75%",
            margin: "20px 0",

            div {
                display: "flex",
                justify_content: "center",
                height: "100%",
                color: "black",
                border_radius: "25px 0 0 25px",
                transition: "width 0.5s",
                line_height: "30px",
                width: "{score}%",
                background_color: "{color}",

                "{score}%"
            }
        }
    }
}

pub fn big_five_bars(scores: Scores, label_top: bool) -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "column",

            PercentileBar { tr: Trait::Open, score: scores.o as u32 , label_top}
            PercentileBar { tr: Trait::Con, score: scores.c as u32 , label_top}
            PercentileBar { tr: Trait::Extro, score: scores.e as u32 , label_top}
            PercentileBar { tr: Trait::Agree, score: scores.a as u32 , label_top}
            PercentileBar { tr: Trait::Neurotic, score: scores.n as u32 , label_top}
        }
    }
}
