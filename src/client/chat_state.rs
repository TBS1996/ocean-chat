#![allow(non_snake_case)]

use crate::common;
use common::ChangeState;
use common::Scores;

use crate::common::UserStatus;
use common::SocketMessage;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use web_sys::WebSocket;

use std::sync::MutexGuard;

use super::*;

#[derive(Default, Clone)]
pub struct ChatState {
    inner: Arc<Mutex<InnerChat>>,
}

impl ChatState {
    pub fn input(&self) -> Signal<String> {
        self.inner().input.clone()
    }

    pub fn messages(&self) -> Signal<Vec<Message>> {
        self.inner().messages.clone()
    }

    pub fn popup(&self) -> Signal<bool> {
        self.inner().popup.clone()
    }

    pub fn status(&self) -> Signal<UserStatus> {
        self.inner().status.clone()
    }

    pub fn send_chat_message(&self, msg: String) -> bool {
        self.inner().send_chat_message(msg)
    }

    pub fn send_info(&mut self, msg: String) {
        self.inner().send_info_message(msg);
    }

    pub fn clear_socket(&self) {
        self.inner().clear_socket();
    }
    pub fn set_peer_scores(&self, scores: Scores) {
        self.inner().peer_scores = Some(scores);
    }
    pub fn set_status(&self, status: UserStatus) {
        self.inner().set_status(status);
    }

    pub fn insert_message(&self, msg: Message) {
        self.inner().messages.write().push(msg);
    }

    pub fn send_message(&self, msg: SocketMessage) -> bool {
        self.inner().send_message(msg)
    }

    pub fn is_disconnected(&self) -> bool {
        self.inner().is_disconnected()
    }

    pub fn peer_scores(&self) -> Option<Scores> {
        self.inner().peer_scores.clone()
    }

    fn inner(&self) -> MutexGuard<'_, InnerChat> {
        self.inner.lock().unwrap()
    }

    pub async fn new_peer(
        &self,
        scores: Scores,
        peer_score_signal: Signal<Option<Scores>>,
        is_disconnected: bool,
    ) -> Result<(), String> {
        let mut lock = self.inner();
        lock.messages.write().clear();
        lock.peer_scores = None;

        let msg = Message::new_info("searching for peer...");
        lock.messages.write().push(msg);
        lock.input.set(String::new());
        if is_disconnected {
            lock.send_message(SocketMessage::GetStatus);
        }

        if is_disconnected {
            let ws = connect_to_peer(scores, self.clone(), peer_score_signal).await?;
            lock.socket = Some(ws);
        } else {
            lock.send_message(SocketMessage::StateChange(ChangeState::Waiting));
        };

        Ok(())
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
    prev_status: Option<UserStatus>,
}

impl InnerChat {
    fn clear_socket(&mut self) {
        log_to_console("clearing socket");

        if let Some(ws) = self.socket.take() {
            let res = ws.close();
            log_to_console(res);
        }

        *self.status.write() = UserStatus::Disconnected;
    }

    fn set_status(&mut self, status: UserStatus) {
        if self.prev_status == Some(UserStatus::Connected) && status == UserStatus::Idle {
            self.send_info_message("Connection with peer lost.".into());
        }

        *self.status.write() = status;
        self.prev_status = Some(status);
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
