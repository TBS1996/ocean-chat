use super::*;

use crate::client;
use client::Route;
use components::nav_bar::top_bar;

#[component]
pub fn Splash() -> Element {
    let navigator = use_navigator();

    rsx! {
    style { { include_str!("css/splash.css") } }
    div {
        display: "flex",
        flex_direction: "column",
        align_items: "center",
        justify_content: "center",
        height: "100vh",

        { top_bar() }

        div {
            class: "content",


            div {
                max_width: "700px",
                display: "flex",
                align_items: "center",
                padding: "10px",
                flex_direction: "column",
                justify_content: "right",

                h2 { "Engage with Similar Minds" }
                p { "{CONTENT}" }
                p{" Give it a go and see if you can find someone ",
                    span {
                        style: "font-style: italic;",
                        "truly"
                    },
                    " like-minded."
                }
            }


            div {
                width: "100%",
                display: "flex",
                justify_content: "left",
                padding: "20px",
                flex_direction: "column",
                align_items: "center",

                div {
                    class: "main-box",
                    onclick: move |_| {navigator.push(Route::Pretest{});},
                    h2 { "Get started!" }
                }
                    }

            }
        }
    }
}

const CONTENT: &'static str = "OceanChat offers a personality-based chat experience. Using the only general personality test taken seriously by researchers, we can scientifically measure how similar you are to anyone else.";
