#![allow(non_snake_case)]
use crate::client;
use crate::common;

use client::log_to_console;
use client::score_cmp;
use client::Navbar;
use client::Splash;
use client::State;
use common::Scores;
use common::SocketMessage;
use common::CONFIG;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

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
        Navbar { active_chat: true },
        if is_init() {
            { enabled_chat(state, input, peer_score, scores, messages, popup.clone()) }
        }
        else {
            { disabled_chat(state, is_init, peer_score, scores, messages, input) }
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
                        MessageList { messages: messages.read().to_vec() }
                    }
                    { form_group(state.clone(), input, peer_score, scores, messages, true ) }
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
                    },
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
                                    p {
                                        "{more_similar}% of people are more similar to you than your peer."
                                    }
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
            div {
                class: "message-list",
                display: "flex",
                max_width: "700px",
                button {
                    class: "mybutton",
                    width: "200px",
                    height: "200px",
                    margin_top: "175px",
                    margin_left: "225px",
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

async fn connect_to_peer(
    scores: Scores,
    state: State,
    peer_score_signal: Signal<Option<Scores>>,
) -> Result<WebSocket, String> {
    log_to_console("Starting to connect");
    let url = format!(
        "{}/pair/{}/{}",
        CONFIG.server_address(),
        scores,
        state.id().simple().to_string()
    );

    // Attempt to create the WebSocket
    let ws = web_sys::WebSocket::new(&url).map_err(|err| {
        let err_msg = format!("Failed to create WebSocket: {:?}", err);
        log_to_console(&err_msg);
        err_msg
    })?;
    log_to_console("WebSocket created");

    // Handle WebSocket open event
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        log_to_console("Connection opened");
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let the_state = state.clone();
    // Handle WebSocket message event
    let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        let state = the_state.clone();
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let txt = txt.as_string().unwrap();

            let message = match serde_json::from_str(&txt).unwrap() {
                SocketMessage::User(msg) => Message::new(Origin::Peer, msg),
                SocketMessage::Info(msg) => Message::new(Origin::Info, msg),
                SocketMessage::Ping => {
                    let msg = SocketMessage::pong();
                    state.send_message(msg);
                    return;
                }
                SocketMessage::Pong => {
                    log_to_console("unexpected pong!");
                    return;
                }
                SocketMessage::ConnectionClosed => {
                    log_to_console("received 'connection closed' from server");
                    state.clear_socket();
                    return;
                }
                SocketMessage::PeerScores(peer_scores) => {
                    log_to_console(("peer score received", &peer_scores));
                    *peer_score_signal.write_unchecked() = Some(peer_scores);
                    state.set_peer_scores(peer_scores);

                    let more_similar = format!("{:.1}", scores.percentage_similarity(peer_scores));
                    let s = format!(
                        "{}% of people are more similar to you than your peer.",
                        more_similar
                    );

                    Message::new(Origin::Info, s)
                }
            };

            state.insert_message(message);

            log_to_console(&format!("Received message: {}", txt));
        }
    }) as Box<dyn FnMut(_)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let the_state = state.clone();
    // Handle WebSocket error event
    let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
        let err_msg = format!(
            "WebSocket error: {:?}, message: {:?}, filename: {:?}, line: {:?}, col: {:?}",
            e,
            e.message(),
            e.filename(),
            e.lineno(),
            e.colno()
        );
        the_state.insert_message(Message::new(
            Origin::Info,
            "unexpected error occured".into(),
        ));
        log_to_console(&err_msg);
    }) as Box<dyn FnMut(_)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // Handle WebSocket close event
    let onclose_callback = Closure::wrap(Box::new(move |_| {
        state.clear_socket();
        log_to_console("WebSocket connection closed");
        state.insert_message(Message::new(Origin::Info, "Connection closed".into()));
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();
    Ok(ws)
}

#[derive(Debug, PartialEq, Clone)]
enum Origin {
    Me,
    Peer,
    Info,
}

impl Origin {
    fn class(&self) -> &'static str {
        match self {
            Self::Me => "message me",
            Self::Peer => "message peer",
            Self::Info => "message info",
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Me => "You: ",
            Self::Peer => "Peer: ",
            Self::Info => "Info: ",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    origin: Origin,
    content: String,
}

impl Message {
    fn new(origin: Origin, content: String) -> Self {
        Self { origin, content }
    }
}

#[derive(Props, PartialEq, Clone)]
struct MessageProps {
    content: String,
    class: &'static str,
    sender: &'static str,
}

fn Message(msg: MessageProps) -> Element {
    rsx!(
        div {
            class: "{msg.class}",
            strong { "{msg.sender}" }
            span { "{msg.content}" }
        }
    )
}

#[derive(Props, PartialEq, Clone)]
struct MessageListProps {
    messages: Vec<Message>,
}

fn MessageList(mut msgs: MessageListProps) -> Element {
    msgs.messages.reverse();
    rsx!(
        div {
            class: "message-list",
            display: "flex",
            flex_direction: "column-reverse",
            for msg in msgs.messages{
                Message {class: msg.origin.class(), sender: msg.origin.str(), content: msg.content}
            }
        }
    )
}
