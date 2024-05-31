#![allow(non_snake_case)]

use crate::client::log_to_console;
use crate::client::Invalid;
use crate::client::State;
use crate::common::Scores;
use crate::common::SocketMessage;
use crate::common::CONFIG;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::WebSocket;

async fn connect_to_peer(
    scores: Scores,
    mut messages: Signal<Vec<Message>>,
    state: State,
) -> Result<WebSocket, String> {
    log_to_console("Starting to connect");
    let url = format!("{}/pair/{}", CONFIG.server_address(), scores);

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
                SocketMessage::PeerScores(peer_scores) => {
                    state.set_peer_scores(peer_scores);
                    let diff = scores.distance(&peer_scores);
                    let msg = format!(
                        "Your peer's personality has a difference-score of {} compared to you!",
                        diff
                    );

                    Message::new(Origin::Info, msg)
                }
            };

            messages.write().push(message);

            log_to_console(&format!("Received message: {}", txt));
        }
    }) as Box<dyn FnMut(_)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

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
        log_to_console(&err_msg);
    }) as Box<dyn FnMut(_)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // Handle WebSocket close event
    let onclose_callback = Closure::wrap(Box::new(move |_| {
        state.clear_socket();
        log_to_console("WebSocket connection closed");
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();
    Ok(ws)
}

#[component]
pub fn Chat() -> Element {
    let state = use_context::<State>();
    let mut input = use_signal(String::new);

    let Some(scores) = state.scores() else {
        return Invalid();
    };

    let mut messages = use_signal(|| {
        vec![Message {
            origin: Origin::Info,
            content: "searching for peer...".to_string(),
        }]
    });

    use_effect({
        let state = state.clone();
        move || {
            let state = state.clone();
            spawn_local(async move {
                let socket = connect_to_peer(scores, messages, state.clone())
                    .await
                    .unwrap();
                state.set_socket(socket);
            });
        }
    });

    let the_state = state.clone();
    let disabled = !state.has_socket();
    rsx! {
        form {
            onsubmit: move |event| {
                let msg = event.data().values().get("msg").unwrap().as_value();
                input.set(String::new());
                let state = the_state.clone();
                messages.write().push(Message::new(Origin::Me, msg.clone()));
                if state.send_message(&msg) {
                    log_to_console("message submitted");
                }
            },
            style { { include_str!("../styles.css") } }
            div {
                class: "chat-app",
                MessageList { messages: messages.read().clone() }
            }
            div { class: "form-group",
                div { class: "input-group",
                    input {
                        r#type: "text",
                        name: "msg",
                        value: "{input}",
                        disabled: "{disabled}",
                        autocomplete: "off",
                        oninput: move |event| input.set(event.value()),
                    }
                    input { r#type: "submit", value: "Submit", disabled: "{disabled}" }
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            messages.write().clear();
                            state.clear_peer();
                            let state = state.clone();
                            spawn_local(async move {
                                let state = state.clone();
                                let socket = connect_to_peer(scores, messages, state.clone())
                                    .await
                                    .unwrap();
                                state.set_socket(socket);
                            });
                            let msg = Message {
                                origin: Origin::Info,
                                content: "searching for peer...".to_string()};
                            messages.write().push(msg);
                            input.set(String::new());
                        },
                        "New peer"
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
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

#[derive(PartialEq, Clone)]
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
