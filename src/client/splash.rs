#![allow(non_snake_case)]

use crate::client;

use client::Route;
use dioxus::prelude::*;

#[component]
pub fn Splash() -> Element {
    let navigator = use_navigator();

    rsx! {
        style { { include_str!("splash.css") } }
        div {
            class: "splash-container",
            h1 { "Welcome to Oceanchat!" }
            p { "Before you can start chatting, we need to know your personality scores." }
            div {
                class: "main-box",
                onclick: move |_| {navigator.push(Route::Test{});},
                h2 {"Take the test!"}

            }
            p {
                class: "small-text",
                "Already know your big-5 scores? Enter them here!"
            }
            div {
                class: "secondary-box",
                onclick: move |_| {navigator.push(Route::Manual{});},
                h2 {"Manual input"}

            }
        }
    }
}
