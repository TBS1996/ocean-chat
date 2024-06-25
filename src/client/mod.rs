#![allow(non_snake_case)]

use crate::common;
use common::Scores;

use dioxus::prelude::*;
use futures::executor::block_on;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

mod chat;
mod getstarted;
mod manual;
mod personality;
mod privacypolicy;
mod splash;
mod test;
pub mod utils;

use chat::*;
use getstarted::*;
use manual::*;
use personality::*;
use privacypolicy::*;
use splash::*;
use test::*;
use utils::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Wrapper)]
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
    #[route("/pretest")]
    Pretest {},
    #[route("/privacypolicy")]
    Privacypolicy {},
}

impl Route {
    fn on_chat(&self) -> bool {
        let state = use_context::<State>();

        // Being 'Home' puts you in chat window if you've already selected your scores.
        if matches!(self, Route::Home {}) && state.scores().is_some() {
            return true;
        }

        matches!(self, Route::Chat {})
    }
}

#[component]
fn Wrapper() -> Element {
    let on_chat_window = use_route::<Route>().on_chat();

    rsx! {
        Outlet::<Route> {}
        div {
            display: "flex",
            justify_content: "center",
            if !on_chat_window {
                { footer() }
            }
        }
    }
}

fn App() -> Element {
    use_context_provider(State::load);
    use_context_provider(Quiz::new);

    rsx!(Router::<Route> {})
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

#[derive(Clone, Default)]
pub struct State {
    inner: Arc<Mutex<InnerState>>,
}

#[derive(Default)]
struct ChatState {
    peer_scores: Option<Scores>,
    socket: Option<WebSocket>,
    messages: Signal<Vec<Message>>,
    input: Signal<String>,
    connected: Signal<bool>,
    popup: Signal<bool>,
    init: bool,
}

#[derive(Default)]
struct InnerState {
    chat: ChatState,
    scores: Option<Scores>,
    user_id: Uuid,
}

impl InnerState {
    fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }
}

impl State {
    pub fn load() -> Self {
        let id = match block_on(fetch_id_storage()) {
            Some(id) => id,
            None => {
                let id = Uuid::new_v4();
                save_id(id);
                id
            }
        };

        let s = Self {
            inner: Arc::new(Mutex::new(InnerState::new(id))),
        };

        if let Some(scores) = block_on(fetch_scores_storage()) {
            log_to_console("score set!");
            s.set_scores(scores);
        } else {
            log_to_console("score not set!");
        };
        s
    }

    pub fn popup(&self) -> Signal<bool> {
        self.inner.lock().unwrap().chat.popup.clone()
    }

    fn is_init(&self) -> bool {
        self.inner.lock().unwrap().chat.init
    }

    fn set_init(&self, init: bool) {
        self.inner.lock().unwrap().chat.init = init;
    }

    pub fn id(&self) -> Uuid {
        self.inner.lock().unwrap().user_id
    }

    pub fn insert_message(&self, message: Message) {
        log_to_console(("inserting msg:", &message));

        self.inner
            .lock()
            .unwrap()
            .chat
            .messages
            .push(message.clone());
    }

    pub fn input(&self) -> Signal<String> {
        self.inner.lock().unwrap().chat.input.clone()
    }

    pub fn messages(&self) -> Signal<Vec<Message>> {
        self.inner.lock().unwrap().chat.messages.clone()
    }

    pub fn clear_messages(&self) {
        self.inner.lock().unwrap().chat.messages.clear();
    }

    pub fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }

    pub fn peer_scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().chat.peer_scores
    }

    pub fn set_peer_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().chat.peer_scores = Some(scores);
    }

    pub fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    pub fn has_socket(&self) -> bool {
        self.inner.lock().unwrap().chat.socket.is_some()
    }

    pub fn set_socket(&self, socket: WebSocket) {
        log_to_console("setting socket");
        self.inner.lock().unwrap().chat.socket = Some(socket);
        *self.not_connected().write() = false;
    }

    fn not_connected(&self) -> Signal<bool> {
        self.inner.lock().unwrap().chat.connected.clone()
    }

    fn clear_socket(&self) {
        log_to_console("clearing socket");

        if let Some(ws) = &self.inner.lock().unwrap().chat.socket {
            let x = ws.close();
            log_to_console(x);
        }

        self.inner.lock().unwrap().chat.socket = None;

        *self.not_connected().write() = true;
    }

    pub fn send_message(&self, msg: Vec<u8>) -> bool {
        if let Some(socket) = &self.inner.lock().unwrap().chat.socket {
            let res = socket.send_with_u8_array(&msg);
            log_to_console(("message sent", res));
            true
        } else {
            log_to_console("attempted to send msg without a socket configured");
            false
        }
    }

    pub fn clear_peer(&self) {
        log_to_console("clear peer");
        let mut lock = self.inner.lock().unwrap();
        if let Some(socket) = &lock.chat.socket {
            socket.close().unwrap();
        }
        lock.chat.peer_scores = None;
        lock.chat.socket = None;
    }
}
