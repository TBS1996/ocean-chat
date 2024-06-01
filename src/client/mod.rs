#![allow(non_snake_case)]

use crate::common;
use chat::Chat;
use chat::Message;
use common::Scores;
use dioxus::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use test::Test;
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::WebSocket;

mod chat;
mod test;

#[wasm_bindgen(start)]
pub fn run_app() {
    launch(App);
}

#[derive(Clone, Default)]
pub struct State {
    inner: Arc<Mutex<InnerState>>,
}

#[derive(Default)]
struct InnerState {
    scores: Option<Scores>,
    peer_scores: Option<Scores>,
    socket: Option<WebSocket>,
    messages: Signal<Vec<Message>>,
    input: Signal<String>,
    connected: Signal<bool>,
}

impl State {
    pub fn insert_message(&self, message: Message) {
        log_to_console("inserting msg");
        log_to_console(&message);
        self.inner.lock().unwrap().messages.push(message);
    }

    pub fn input(&self) -> Signal<String> {
        self.inner.lock().unwrap().input.clone()
    }

    pub fn messages(&self) -> Signal<Vec<Message>> {
        self.inner.lock().unwrap().messages.clone()
    }

    pub fn clear_messages(&self) {
        self.inner.lock().unwrap().messages.clear();
    }

    pub fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }

    pub fn set_peer_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().peer_scores = Some(scores);
    }

    pub fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    pub fn has_socket(&self) -> bool {
        self.inner.lock().unwrap().socket.is_some()
    }

    pub fn set_socket(&self, socket: WebSocket) {
        self.inner.lock().unwrap().socket = Some(socket);
        *self.not_connected().write() = false;
    }

    fn not_connected(&self) -> Signal<bool> {
        self.inner.lock().unwrap().connected.clone()
    }

    fn clear_socket(&self) {
        self.inner.lock().unwrap().socket = None;
        *self.not_connected().write() = true;
    }

    pub fn send_message(&self, msg: &str) -> bool {
        if let Some(socket) = &self.inner.lock().unwrap().socket {
            let _ = socket.send_with_str(msg);
            true
        } else {
            log_to_console("attempted to send msg without a socket configured");
            false
        }
    }

    pub fn clear_peer(&self) {
        let mut lock = self.inner.lock().unwrap();
        if let Some(socket) = &lock.socket {
            socket.close().unwrap();
        }
        lock.peer_scores = None;
        lock.socket = None;
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/invalid")]
    Invalid {},
    #[route("/chat")]
    Chat {},
    #[route("/test")]
    Test {},
    #[route("/manual")]
    Manual {},
}

fn App() -> Element {
    use_context_provider(State::default);
    rsx!(Router::<Route> {})
}

// Call this function to log a message
pub fn log_to_console(message: impl std::fmt::Debug) {
    let message = format!("{:?}", message);
    console::log_1(&JsValue::from_str(&message));
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        nav {
            class: "sidebar",
            ul {
                li {
                    Link { to: Route::Home {}, "Home" }
                }

                li {
                    Link { to: Route::Manual {}, "Enter scores manually" }
                }

                li {
                    Link { to: Route::Test {}, "Take the personality test" }
                }

                li {
                    Link { to: Route::Chat {}, "Start chatting" }
                }
            }
        }
    }
}

#[component]
fn Manual() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();

    rsx! {
        style { { include_str!("../styles.css") } }
        main {
            class: "layout",
            Sidebar {},
            div {
                h1 {"Manual Scores"}
                br {}
                form {
                    onsubmit:  move |event| {
                        match Scores::try_from(event.data().deref()) {
                            Ok(scores) => {
                                state.set_scores(scores);
                                navigator.replace(Route::Chat{});
                            }
                            Err(_) => {
                                navigator.replace(Route::Invalid {});
                            }
                        }
                    },

                    div {
                        class: "spread-around",
                        label { r#for: "o", "Openness: " }
                        input { id: "o", name: "o", value: "50", r#type: "number" }
                    }
        
                    div {
                        class: "spread-around",
                        label { r#for: "c", "Conscientiousness: " }
                        input { id: "c", name: "c", value: "50", r#type: "number" }
                    }
        
                    div {
                        class: "spread-around",
                        label { r#for: "e", "Extraversion: " }
                        input { id: "e", name: "e", value: "50", r#type: "number" }
                    }
                    
                    div {
                        class: "spread-around",
                        label { r#for: "a", "Agreeableness: " }
                        input { id: "a", name: "a", value: "50", r#type: "number" }
                    }
        
                    div {
                        class: "spread-around",
                        label { r#for: "n", "Neuroticism: " }
                        input { id: "n", name: "n", value: "50", r#type: "number" }
                    }
        
                    br {}
                    
                    button {
                        class: "confirm",
                        r#type: "submit",
                        h2 { "Save" }
                    }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        style {
            { include_str!("../styles.css") }
        }
        main {
            class: "layout",
            Sidebar {},
            div {
                h1 { "Hello! Welcome to Oceanchat!" }
                p {
                    "Start chatting with people similar to your personality here.
                    First you must take the personality test, or manually input your Big 5 trait scores!"
                }
            }
        }
    }
}

#[component]
pub fn Invalid() -> Element {
    rsx! {
        div {
            p {
                "You have to either take the personality test, or manually submit a valid set of trait scores!"
            }
            div {
                Link {
                    to: Route::Home {},
                    "Back to main page"
                }
            }
        }
    }
}
