#![allow(non_snake_case)]

use crate::common;
use common::ChangeState;
use common::Scores;

use dioxus::prelude::*;
use futures::executor::block_on;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

mod components;
mod pages;
pub mod utils;

use pages::chat::*;
use pages::getstarted::*;
use pages::manual::*;
use pages::personality::*;
use pages::privacypolicy::*;
use pages::splash::*;
use pages::test::*;
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

use crate::common::UserStatus;
use common::SocketMessage;

#[derive(Default, Clone)]
pub struct ChatState {
    inner: Arc<Mutex<InnerChat>>,
}

impl ChatState {
    pub fn send_chat_message(&mut self, msg: String) -> bool {
        self.inner.lock().unwrap().send_chat_message(msg)
    }

    pub fn send_info(&mut self, msg: String) {
        self.inner.lock().unwrap().send_info_message(msg);
    }

    pub fn clear_socket(&self) {
        self.inner.lock().unwrap().clear_socket();
    }
    pub fn set_peer_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().peer_scores = Some(scores);
    }
    pub fn set_status(&self, status: UserStatus) {
        self.inner.lock().unwrap().set_status(status);
    }

    pub fn insert_message(&self, msg: Message) {
        self.inner.lock().unwrap().messages.write().push(msg);
    }

    pub fn send_message(&self, msg: SocketMessage) -> bool {
        self.inner.lock().unwrap().send_message(msg)
    }

    pub fn is_disconnected(&self) -> bool {
        self.inner.lock().unwrap().is_disconnected()
    }
}

#[derive(Default)]
struct InnerChat {
    messages: Signal<Vec<Message>>,
    peer_scores: Option<Scores>,
    status: Signal<UserStatus>,
    socket: Option<WebSocket>,
    input: Signal<String>,
    popup: Signal<bool>,
    the_status: Option<UserStatus>,
}

impl InnerChat {
    fn clear_socket(&mut self) {
        log_to_console("clearing socket");

        if let Some(ws) = self.socket.take() {
            let x = ws.close();
            log_to_console(x);
        }

        *self.status.write() = UserStatus::Disconnected;
    }

    fn set_status(&mut self, status: UserStatus) {
        if self.the_status == Some(UserStatus::Connected) && status == UserStatus::Idle {
            self.send_info_message("Connection with peer lost.".into());
        }

        *self.status.write() = status;
        self.the_status = Some(status);
    }

    pub fn send_info_message(&mut self, msg: String) {
        let msg = Message::new_info(msg);
        self.messages.write().push(msg);
    }

    pub fn send_chat_message(&mut self, msg: String) -> bool {
        let data = SocketMessage::User(msg.clone());
        let msg = Message::new_from_me(msg);
        self.messages.write().push(msg);
        self.reset_input();
        self.send_message(data)
    }

    pub fn send_message(&self, msg: SocketMessage) -> bool {
        let msg = msg.to_bytes();
        if let Some(socket) = &self.socket {
            let res = socket.send_with_u8_array(&msg);
            log_to_console(("message sent", res));
            true
        } else {
            log_to_console("attempted to send msg without a socket configured");
            false
        }
    }

    fn is_disconnected(&self) -> bool {
        *self.status.read() == UserStatus::Disconnected
    }

    fn reset_input(&mut self) {
        self.input.set(String::new());
    }
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

pub fn get_id() -> Uuid {
    match block_on(fetch_id_storage()) {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4();
            save_id(id);
            id
        }
    }
}

impl State {
    pub fn load() -> Self {
        let id = get_id();
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

    pub async fn new_peer(
        &self,
        scores: Scores,
        peer_score_signal: Signal<Option<Scores>>,
        is_disconnected: bool,
        id: Uuid,
    ) -> Result<(), String> {
        let chat = self.inner.lock().unwrap().chat.clone();
        let mut lock = chat.inner.lock().unwrap();

        lock.messages.write().clear();
        lock.peer_scores = None;

        let msg = Message::new_info("searching for peer...");
        lock.messages.write().push(msg);
        lock.input.set(String::new());
        if is_disconnected {
            lock.send_message(SocketMessage::GetStatus);
        }

        if is_disconnected {
            let ws = connect_to_peer(scores, chat.clone(), peer_score_signal, id).await?;
            lock.socket = Some(ws);
        } else {
            lock.send_message(SocketMessage::StateChange(ChangeState::Waiting));
        };

        drop(lock);

        Ok(())
    }

    pub fn popup(&self) -> Signal<bool> {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .popup
            .clone()
    }

    pub fn id(&self) -> Uuid {
        self.inner.lock().unwrap().user_id
    }

    pub fn clear_socket(&self) {
        self.inner.lock().unwrap().chat.clear_socket();
    }

    pub fn send_chat_message(&self, msg: String) -> bool {
        self.inner.lock().unwrap().chat.send_chat_message(msg)
    }
    pub fn send_socket_message(&self, msg: SocketMessage) -> bool {
        self.inner.lock().unwrap().chat.send_message(msg)
    }

    pub fn insert_message(&self, message: Message) {
        log_to_console(("inserting msg:", &message));

        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .messages
            .push(message.clone());
    }

    pub fn input(&self) -> Signal<String> {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .input
            .clone()
    }

    pub fn messages(&self) -> Signal<Vec<Message>> {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .messages
            .clone()
    }

    pub fn clear_messages(&self) {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .messages
            .clear();
    }

    pub fn set_status(&self, status: UserStatus) {
        let lock = self.inner.lock().unwrap();

        if *lock.chat.inner.lock().unwrap().status.read() != UserStatus::Connected
            && status == UserStatus::Connected
        {
            self.insert_message(Message::new_info("connected!"));
        }

        lock.chat.set_status(status);
    }

    pub fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }

    pub fn peer_scores(&self) -> Option<Scores> {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .peer_scores
    }

    pub fn set_peer_scores(&self, scores: Scores) {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .peer_scores = Some(scores);
    }

    pub fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    pub fn has_socket(&self) -> bool {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .socket
            .is_some()
    }

    pub fn set_socket(&self, socket: WebSocket) {
        log_to_console("setting socket");
        self.inner.lock().unwrap().chat.inner.lock().unwrap().socket = Some(socket);
    }

    fn status(&self) -> Signal<UserStatus> {
        self.inner
            .lock()
            .unwrap()
            .chat
            .inner
            .lock()
            .unwrap()
            .status
            .clone()
    }
}
