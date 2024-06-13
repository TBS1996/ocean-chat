#![allow(non_snake_case)]

use crate::client;

use client::markdown_converter;
use client::top_bar;
use client::Navbar;
use client::State;
use dioxus::prelude::*;

#[component]
pub fn Privacypolicy() -> Element {
    let state = use_context::<State>();
    let show_sidebar = state.scores().is_some();
    let policy = include_str!("../../files/privacypolicy.md");

    rsx! {
        if show_sidebar {Navbar {active_chat: false}} else { { top_bar() } },
        { markdown_converter(policy) }
    }
}
