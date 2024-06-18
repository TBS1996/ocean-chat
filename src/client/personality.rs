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
    let sloan = format!("{:?}", sloan).to_uppercase();
    let weirdness = scores.weirdness_percent() as u32;

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "95vh",
        Navbar{active_chat: false}
        div {
            class: "navmargin",

            style { { include_str!("personality.css") } },

            div {
                max_width: "800px",
                margin: "auto",
                padding: "20px",
                font_family: "Arial, sans-serif",

                h1 { "{sloan}" }
                { big_five_bars(scores, false) }
                div {
                    display: "flex",
                    flex_direction: "row",
                    justify_content: "center",
                    margin_top: "25px",
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
                h2 {"You are weirder than {weirdness}% of the general population!"},

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
fn trait_bar(tr: Trait, score: u32, label_top: bool) -> Element {
    let low_type = tr.low_type();
    let high_type = tr.high_type();

    let color = tr.color();
    let score_position = (score as f64 / 100.0) * 100.0; // Now based on percentage

    let left_weight = if score < 50 { "bold" } else { "normal" };
    let right_weight = if score >= 50 { "bold" } else { "normal" };

    rsx! {
        div {
            display: "flex",
            align_items: "center",
            flex_direction: "column",

            div {margin_top: "15px", "{tr}: {score}%"},
            div {
                display: "flex",
                flex_direction: "row",
                justify_content: "space-between",
                width: "100%", // Full width

                div {
                    padding_right: "10px",
                    text_align: "right",
                    width: "100px",
                    font_weight: "{left_weight}",
                    "{low_type}"
                }

                div {
                    display: "flex",
                    justify_content: "center",
                    position: "relative",
                    height: "30px",
                    width: "calc(100% - 220px)", // 100% minus side labels
                    background_color: "{color}",

                    div {
                        position: "absolute",
                        left: "{score_position}%",
                        height: "100%",
                        width: "10px",
                        background_color: "black",
                    }
                }

                div {
                    padding_left: "10px",
                    text_align: "left",
                    width: "100px",
                    font_weight: "{right_weight}",
                    "{high_type}"
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

            trait_bar { tr: Trait::Extro, score: scores.e as u32 , label_top}
            trait_bar { tr: Trait::Neurotic, score: scores.n as u32 , label_top}
            trait_bar { tr: Trait::Con, score: scores.c as u32 , label_top}
            trait_bar { tr: Trait::Agree, score: scores.a as u32 , label_top}
            trait_bar { tr: Trait::Open, score: scores.o as u32 , label_top}
        }
    }
}

