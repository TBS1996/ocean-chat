#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::common;
use common::Scores;

use dioxus::prelude::*;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::console;
use web_sys::WebSocket;

mod chat;
mod personality;
mod splash;
mod test;

use chat::*;
use personality::*;
use splash::*;
use test::*;

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
    pub fn load() -> Self {
        let s = Self::default();
        if let Some(scores) = block_on(fetch_scores_storage()) {
            log_to_console("score set!");
            s.set_scores(scores);
        } else {
            log_to_console("score not set!");
        };
        s
    }

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
    #[route("/splash")]
    Splash {},
    #[route("/personality")]
    Personality {},
}

fn App() -> Element {
    use_context_provider(State::load);
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
       // style { { include_str!("../styles.css") } },
        div {
            class: "sidebar",
            ul {
                 li {
                    Link { to: Route::Chat {}, "Chat" }
                }
                 li {
                    Link { to: Route::Personality {}, "My personality" }
                }
            }
        }
    }
}

fn default_scores() -> Scores {
    static COOKIE: Lazy<Option<Scores>> = Lazy::new(|| {
        let scores = block_on(fetch_scores_storage());
        scores
    });

    COOKIE.unwrap_or_else(Scores::mid)
}

async fn fetch_scores_storage() -> Option<Scores> {
    let mut eval = eval(
        r#"
        let scores = localStorage.getItem('scores');
        if (scores) {
            dioxus.send(scores);
        } else {
            dioxus.send(null);
        }
        "#,
    );

    let scores = eval.recv().await.unwrap().to_string();
    log_to_console(&scores);
    Scores::from_str(&scores).ok()
}

#[component]
fn Manual() -> Element {
    let navigator = use_navigator();
    let state = use_context::<State>();
    let score = default_scores();

    rsx! {
            style { { include_str!("../styles.css") } }
        div {
            class: "layout",
            Sidebar {},
            div {
            form { onsubmit:  move |event| {
                 match Scores::try_from(event.data().deref()) {
                     Ok(scores) => {
                         state.set_scores(scores);
                         save_scores(scores);
                         navigator.replace(Route::Personality{});
                     }
                     Err(_) => {
                         navigator.replace(Route::Invalid {});
                     }

                 }

            },
    div { class: "form-group",
                label { "Openness: " }
                input { name: "o", value: "{score.o}"}
                }
                div { class: "form-group",
                    label { "Conscientiousness: " }
                    input { name: "c" , value: "{score.c}"}
                }
                div { class: "form-group",
                    label { "Extraversion: " }
                    input { name: "e", value: "{score.e}"}
                }
                div { class: "form-group",
                    label { "Agreeableness: " }
                    input { name: "a" , value: "{score.a}"}
                }
                div { class: "form-group",
                    label { "Neuroticism: " }
                    input { name: "n", value: "{score.n}"}
                }
                div { class: "form-group",
                    input { r#type: "submit", value: "Save" }
            }
        }
            }
    }
    }
}

#[component]
fn Home() -> Element {
    let state = use_context::<State>();

    if state.scores().is_some() {
        return Chat();
    } else {
        return Splash();
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

pub fn save_scores(scores: Scores) {
    let script = format!("localStorage.setItem('scores', '{}');", scores);
    eval(&script);
    log_to_console("storing scores in local storage");
}
