#![allow(non_snake_case)]
use crate::client;
use crate::common;

use client::log_to_console;
use client::Invalid;
use client::Sidebar;
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

async fn connect_to_peer(scores: Scores, state: State) -> Result<WebSocket, String> {
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
                    state.clear_socket();
                    return;
                }
                SocketMessage::PeerScores(peer_scores) => {
                    state.set_peer_scores(peer_scores);
                    let diff = scores.percentage_similarity(peer_scores);
                    let msg = format!(
                        "{:.1}% of people are more similar to you than your peer",
                        diff
                    );

                    Message::new(Origin::Info, msg)
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

#[component]
pub fn Chat() -> Element {
    let state = use_context::<State>();
    let mut input = state.input();
    let mut messages = state.messages();

    let Some(scores) = state.scores() else {
        return Splash();
    };

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
                    messages.write().insert(0, msg);
                    let socket = connect_to_peer(scores, state.clone()).await.unwrap();
                    state.set_socket(socket);
                }
            });
        }
    });

    let state2 = state.clone();

    rsx! {
        main {
            Sidebar { active_chat: true },
            div {
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
                    style { { include_str!("../styles.css") } }
                    div {
                        class: "chat-app",
                        MessageList { messages: messages.read().to_vec() }
                    }
                    div { class: "form-group",
                        div { class: "input-group",
                            input {
                                r#type: "text",
                                name: "msg",
                                value: "{input}",
                                autocomplete: "off",
                                oninput: move |event| input.set(event.value()),
                            }
                            button {
                                r#type: "submit",
                                class: "confirm",
                                "Send"
                            }
                            button {
                                prevent_default: "onclick",
                                class: "danger",
                                onclick: move |_| {
                                    let thestate = state.clone();
                                    messages.write().clear();
                                    state.clear_peer();
                                    spawn_local(async move {
                                        let socket = connect_to_peer(scores, thestate.clone())
                                            .await
                                            .unwrap();
                                        thestate.set_socket(socket);
                                    });
                                    let msg = Message {
                                        origin: Origin::Info,
                                        content: "searching for peer...".to_string()};
                                    messages.write().insert(0, msg);
                                    input.set(String::new());
                                },
                                "New peer"
                            }
                        }
                    }
                }
            }
        }
    }
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

fn MessageList(msgs: MessageListProps) -> Element {
    rsx!(
        div {
            class: "message-list",
            for msg in msgs.messages {
                Message {class: msg.origin.class(), sender: msg.origin.str(), content: msg.content}
            }
        }
    )
}
