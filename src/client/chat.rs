#![allow(non_snake_case)]
use crate::client;
use crate::common;

use client::connect_to_peer;
use client::log_to_console;
use client::score_cmp;
use client::Message;
use client::MessageList;
use client::Navbar;
use client::Origin;
use client::Splash;
use client::State;
use common::Scores;
use common::SocketMessage;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

use once_cell::sync::Lazy;
use std::sync::atomic;
use std::sync::Arc;

pub static PINGER_ACTIVATED: Lazy<Arc<atomic::AtomicBool>> =
    Lazy::new(|| Arc::new(atomic::AtomicBool::new(false)));

fn start_pinger(state: State) {
    spawn_local(async move {
        if PINGER_ACTIVATED.swap(true, atomic::Ordering::SeqCst) {
            return;
        }

        log_to_console("Start pinging loop");
        loop {
            log_to_console("pinging server");
            state.send_message(SocketMessage::ping());
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

    let input = state.input();
    let messages = state.messages();
    let is_init = state.is_init();
    let is_init = use_signal(move || is_init);
    log_to_console(("chat refresh, is_init: ", &is_init));
    let peer_score = use_signal(|| state.peer_scores());
    let popup = state.popup();

    log_to_console(&popup);

    start_pinger(state.clone());

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",
            height: "95vh",

            Navbar { active_chat: true },
            div {
                class: "navmargin",

                if is_init() {
                    { enabled_chat(state, input, peer_score, scores, messages, popup.clone()) }
                }
                else {
                    { disabled_chat(state, is_init, peer_score, scores, messages, input) }
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
    mut messages: Signal<Vec<Message>>,
    enabled: bool,
) -> Element {
    rsx! {
        div { class: "form-group",
            margin_bottom: "30px",

            div { class: "input-group",
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
                        messages.write().clear();
                        state.clear_peer();
                        spawn_local(async move {
                            let socket = connect_to_peer(scores, thestate.clone(), peer_score.clone())
                                .await
                                .unwrap();
                            thestate.set_socket(socket);
                        });
                        let msg = Message {
                            origin: Origin::Info,
                            content: "searching for peer...".to_string()};
                        messages.write().push(msg);
                        input.set(String::new());
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
    mut input: Signal<String>,
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
                        let msg = event.data().values().get("msg").unwrap().as_value();
                        input.set(String::new());
                        state.insert_message(Message::new(Origin::Me, msg.clone()));
                        let msg = SocketMessage::user_msg(msg);
                        if state.send_message(msg) {
                            log_to_console("message submitted");
                        }
                    },
                    div {
                        { form_group(state.clone(), input, peer_score, scores, messages, true) }
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
    mut is_init: Signal<bool>,
    peer_score: Signal<Option<Scores>>,
    scores: Scores,
    mut messages: Signal<Vec<Message>>,
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
                        is_init.toggle();
                        state.set_init(true);
                        use_effect({
                            let state = state.clone();
                            move || {
                                let state = state.clone();
                                spawn_local(async move {
                                    if !state.has_socket() {
                                        let msg = Message {
                                            origin: Origin::Info,
                                            content: "searching for peer...".to_string(),
                                        };
                                        messages.write().push(msg);
                                        let socket = connect_to_peer(scores, state.clone(), peer_score.clone()).await.unwrap();
                                        state.set_socket(socket);
                                    }
                                });
                            }
                        });
                    },
                    "Start chatting!"
                }
            }

            { form_group(state.clone(), input, peer_score, scores, messages, false ) }

        }
    }
}
