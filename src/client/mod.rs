#![allow(non_snake_case)]

use crate::common;
use common::Scores;

use dioxus::prelude::*;
use futures::executor::block_on;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

mod chat_state;
mod components;
mod pages;
pub mod utils;

use chat_state::*;
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
        Some(id) => {
            log_to_console(("using id from storage:", &id));
            id
        }
        None => {
            let id = Uuid::new_v4();
            save_id(id);
            log_to_console(("generated new id:", &id));
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

    pub fn chat(&self) -> ChatState {
        self.inner.lock().unwrap().chat.clone()
    }

    pub fn id(&self) -> Uuid {
        self.inner.lock().unwrap().user_id
    }

    pub fn scores(&self) -> Option<Scores> {
        self.inner.lock().unwrap().scores
    }

    pub fn set_scores(&self, scores: Scores) {
        self.inner.lock().unwrap().scores = Some(scores);
    }
}
