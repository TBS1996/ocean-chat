#![allow(non_snake_case)]

use crate::common::Scores;
use crate::common::SocketMessage;
use crate::common::CONFIG;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use web_sys::WebSocket;

#[wasm_bindgen(start)]
pub fn run_app() {
    launch(App);
}

#[derive(Clone, Default)]
struct State {
    inner: Arc<Mutex<InnerState>>,
}

#[derive(Default)]
struct InnerState {
    scores: Option<Scores>,
    socket: Option<WebSocket>,
}

impl State {
    fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }

    fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    fn set_socket(&self, socket: WebSocket) {
        self.inner.lock().unwrap().socket = Some(socket);
    }

    fn send_message(&self, msg: &str) -> bool {
        if let Some(socket) = &self.inner.lock().unwrap().socket {
            let _ = socket.send_with_str(msg);
            true
        } else {
            log_to_console("attempted to send msg without a socket configured");
            false
        }
    }
}

fn scores_from_formdata(form: &FormData) -> Option<Scores> {
    let data = form.values();

    let o: f32 = data.get("o")?.as_value().parse().ok()?;
    let c: f32 = data.get("c")?.as_value().parse().ok()?;
    let e: f32 = data.get("e")?.as_value().parse().ok()?;
    let a: f32 = data.get("a")?.as_value().parse().ok()?;
    let n: f32 = data.get("n")?.as_value().parse().ok()?;

    if !(0. ..=100.).contains(&o) {
        return None;
    }
    if !(0. ..=100.).contains(&c) {
        return None;
    }
    if !(0. ..=100.).contains(&e) {
        return None;
    }
    if !(0. ..=100.).contains(&a) {
        return None;
    }
    if !(0. ..=100.).contains(&n) {
        return None;
    }

    Some(Scores { o, c, e, a, n })
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/invalid")]
    Invalid {},
    #[route("/chat")]
    Chat {},
}

async fn connect_to_peer(
    scores: Scores,
    mut messages: Signal<Vec<Message>>,
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

    // Handle WebSocket message event
    let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let txt = txt.as_string().unwrap();

            let message = match serde_json::from_str(&txt).unwrap() {
                SocketMessage::User(msg) => Message::new(Origin::Peer, msg),
                SocketMessage::Info(msg) => Message::new(Origin::Info, msg),
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
        log_to_console("WebSocket connection closed");
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    log_to_console("Returning WebSocket");
    Ok(ws)
}

#[component]
fn Chat() -> Element {
    let state = use_context::<State>();

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
                let sock = connect_to_peer(scores, messages).await.unwrap();
                state.set_socket(sock);
            });
        }
    });

    rsx! {
            form { onsubmit:  move | event| {
                let x = event.data().values().get("msg").unwrap().as_value();
                messages.write().push(Message::new(Origin::Me, x.clone()));
                if state.send_message(&x) {
                    log_to_console("message submitted");
                }
            },


        style { { include_str!("./styles.css") } }
        div {
            class: "chat-app",
            MessageList { messages: messages.read().clone() }
        }



    div { class: "form-group",
                    label { "chat msg" }
                    input { name: "msg" }
                }
                div { class: "form-group",
                    input { r#type: "submit", value: "Submit" }
                }




            }
        }
}

fn App() -> Element {
    use_context_provider(State::default);
    rsx!(Router::<Route> {})
}

// Call this function to log a message
fn log_to_console(message: &str) {
    console::log_1(&JsValue::from_str(message));
}

#[component]
fn Invalid() -> Element {
    rsx! {
        "invalid input! all values must be between 0 and 100",
        Link { to: Route::Home {}, "try again" }
    }
}

#[component]
fn Home() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();

    rsx! {
    form { onsubmit:  move |event| {
         match scores_from_formdata(&event.data()) {
             Some(scores) => {
                 state.set_scores(scores);
                 navigator.replace(Route::Chat{});
             }
             None => {
                 navigator.replace(Route::Invalid {});
             }

         }

    },
    div { class: "form-group",
                label { "Openness: " }
                input { name: "o", value: "50"}
                }
                div { class: "form-group",
                    label { "Conscientiousness: " }
                    input { name: "c" , value: "50"}
                }
                div { class: "form-group",
                    label { "Extraversion: " }
                    input { name: "e", value: "50"}
                }
                div { class: "form-group",
                    label { "Agreeableness: " }
                    input { name: "a" , value: "50"}
                }
                div { class: "form-group",
                    label { "Neuroticism: " }
                    input { name: "n", value: "50"}
                }
                div { class: "form-group",
                    input { r#type: "submit", value: "Submit" }
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
