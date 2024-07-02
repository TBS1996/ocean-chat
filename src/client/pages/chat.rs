#![allow(non_snake_case)]
use crate::client;
use crate::client::components::nav_bar::Navbar;
use crate::client::Splash;
use crate::common;

use client::log_to_console;
use client::score_cmp;
use client::Message;
use client::MessageList;
use client::State;
use common::Scores;
use common::SocketMessage;
use common::UserStatus;
use common::CONFIG;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use once_cell::sync::Lazy;
use std::sync::atomic;
use std::sync::Arc;

pub async fn get_status(state: &State) -> Option<UserStatus> {
    use futures::future::{select, Either};
    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let url = format!(
        "{}/status/{}",
        CONFIG.http_address(),
        state.id().simple().to_string()
    );

    let opts = {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);
        opts
    };

    let request = Request::new_with_str_and_init(&url, &opts).ok()?;

    let fetch_future = {
        let window = web_sys::window()?;
        JsFuture::from(window.fetch_with_request(&request))
    };

    let result = select(fetch_future, TimeoutFuture::new(5000)).await;

    match result {
        Either::Left((fetch_result, _)) => {
            let resp: Response = fetch_result.ok()?.dyn_into().ok()?;
            let text = JsFuture::from(resp.text().ok()?).await.ok()?.as_string()?;
            let status: UserStatus = serde_json::from_str(&text).ok()?;
            Some(status)
        }
        Either::Right((_, _)) => return None,
    }
}

pub static PINGER_ACTIVATED: Lazy<Arc<atomic::AtomicBool>> =
    Lazy::new(|| Arc::new(atomic::AtomicBool::new(false)));

fn start_pinger(state: State) {
    let chat = state.chat();
    spawn_local(async move {
        if PINGER_ACTIVATED.swap(true, atomic::Ordering::SeqCst) {
            return;
        }

        log_to_console("Start pinging loop");
        loop {
            chat.send_message(SocketMessage::GetStatus);
            gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

#[component]
pub fn Chat() -> Element {
    let state = use_context::<State>();
    let Some(scores) = state.scores() else {
        return Splash();
    };

    let chat = state.chat();

    let input = chat.input();
    let messages = chat.messages();
    let peer_score = use_signal(|| chat.peer_scores());
    let popup = chat.popup();
    let status = chat.status();

    log_to_console(&popup);

    start_pinger(state.clone());

    let is_enabled = status() != UserStatus::Disconnected;
    log_to_console(("is enabled: ", is_enabled));

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "95vh",

            Navbar { active_chat: true },
            div {
                class: "navmargin",

                if is_enabled {
                    { enabled_chat(state, input, peer_score, scores, messages, popup.clone()) }
                }
                else {
                    { disabled_chat(state, peer_score, scores, input) }
                }
            }
        }
    }
}

fn form_group(
    state: State,
    mut input: Signal<String>,
    peer_score: Signal<Option<Scores>>,
    scores: Scores,
    enabled: bool,
) -> Element {
    rsx! {
        div {
            class: "form-group",
            margin_bottom: "30px",

            div {
                class: "input-group",
                input {
                    r#type: "text",
                    name: "msg",
                    value: input(),
                    disabled: !enabled,
                    autocomplete: "off",
                    background_color: if !enabled {"gray"} else {"white"},
                    border_color: if !enabled {"gray"} else {"white"},
                    oninput: move |event| input.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "confirm",
                    background_color: if !enabled {"gray"} else {""},
                    "Send"
                }
                button {
                    prevent_default: "onclick",
                    class: "danger",
                    onclick: move |_| {
                        if !enabled {
                            return;
                        }

                        let thestate = state.clone();
                        let chat = thestate.chat();
                        let id = thestate.id();
                        let status = chat.status();
                        let is_disconnected = status() == UserStatus::Disconnected;
                        spawn_local(async move {
                            chat.new_peer(scores, peer_score.clone(), is_disconnected, id).await.unwrap();
                        });
                    },
                    background_color: if !enabled {"gray"} else {""},
                    "New peer"
                }
            }
        }
    }
}

fn enabled_chat(
    state: State,
    input: Signal<String>,
    peer_score: Signal<Option<Scores>>,
    scores: Scores,
    messages: Signal<Vec<Message>>,
    mut popup: Signal<bool>,
) -> Element {
    let state2 = state.clone();

    rsx! {
        if !popup() {
            div {
                display: "flex",
                margin_left: "20px",
                max_width: "700px",
                flex_direction: "column",
                position: "relative",
                height: "calc(98vh - 50px)",
                max_height: "600px",
                if peer_score().is_some() {
                    button {
                        position: "absolute",
                        top: "5px",
                        left: "50%",
                        transform: "translateX(-50%)",
                        z_index: "10",
                        prevent_default: "onclick",
                        class: "mybutton",
                        onclick: move |event| {
                            event.stop_propagation();
                            log_to_console("overlay clicked");
                            *popup.write() = true;
                        },
                        "Compare scores"
                    }
                }
                div {
                    display: "flex",
                    flex_direction: "column",
                    flex_grow: "1",
                    overflow_y: "auto",
                    div {
                        flex_grow: "1",
                        display: "flex",
                        flex_direction: "column-reverse",
                        MessageList { messages: messages.read().to_vec() }
                    }
                }
                form {
                    onsubmit: move |event| {
                        let state = state2.clone();
                        let  chat = state.chat();
                        let msg = event.data().values().get("msg").unwrap().as_value();
                        chat.send_chat_message(msg);
                    },
                    div {
                        { form_group(state.clone(), input, peer_score, scores , true) }
                    }
                }
            }
        } else {
            div {
                display: "flex",
                flex_direction: "column",
                margin_left: "20px",
                width: "700px",
                div {
                    display: "flex",
                    justify_content: "center",
                    margin_bottom: "50px",
                    button {
                        width: "250px",
                        prevent_default: "onclick",
                        class: "mybutton",
                        onclick: move |event| {
                            event.stop_propagation();
                            log_to_console("go back clicked");
                            *popup.write() = false;
                        },
                        "go back!"
                    }
                }
                div {
                    width: "500px",
                    margin_left: "100px",
                    match peer_score() {
                        Some(score) => {
                            let more_similar = format!("{:.1}", scores.percentage_similarity(score));
                            rsx! {
                                div {
                                    h4 { "Your peer's personality:" }
                                    { score_cmp(scores, score) }
                                    p { "{more_similar}% of people are more similar to you than your peer." }
                                }
                            }
                        },
                        None => { rsx!{""} },
                    }
                }
            }
        }
    }
}

fn disabled_chat(
    state: State,
    peer_score: Signal<Option<Scores>>,
    scores: Scores,
    input: Signal<String>,
) -> Element {
    rsx! {
        div {
            display: "flex",
            margin_left: "20px",
            max_width: "700px",
            flex_direction: "column",
            position: "relative",
            height: "calc(98vh - 50px)",
            max_height: "600px",

            div {
                class: "message-list",
                display: "flex",
                justify_content: "center",
                width: "100%",
                button {
                    class: "mybutton",
                    width: "200px",
                    height: "200px",
                    margin: "auto",
                    onclick: move |_| {
                                let state = state.clone();
                                let chat = state.chat();
                                let mut status = chat.status();
                                let id = state.id();
                                let is_disconnected = status() == UserStatus::Disconnected;
                                *status.write() = UserStatus::Waiting;

                                spawn_local(async move {
                                    chat.new_peer(scores, peer_score.clone(), is_disconnected, id).await.unwrap();
                                });
                    },
                    "Start chatting!"
                }
            }

            { form_group(state.clone(), input, peer_score, scores, false ) }
        }
    }
}
