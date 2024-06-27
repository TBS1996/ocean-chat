#![allow(non_snake_case)]

use crate::common;

use common::Scores;

use crate::client::Route;

use crate::client::State;
use common::SocketMessage;
use dioxus::prelude::*;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use std::str::FromStr;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::console;

use common::CONFIG;
use wasm_bindgen::JsCast;
use web_sys::WebSocket;

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

pub fn save_id(id: Uuid) {
    let script = format!("localStorage.setItem('user_id', '{}');", id);
    eval(&script);
    log_to_console("storing user_id in local storage");
}

pub fn save_scores(scores: Scores) {
    let script = format!("localStorage.setItem('scores', '{}');", scores);
    eval(&script);
    log_to_console("storing scores in local storage");
}

// Call this function to log a message
pub fn log_to_console(message: impl std::fmt::Debug) {
    let message = format!("{:?}", message);
    console::log_1(&JsValue::from_str(&message));
}

fn default_scores() -> Scores {
    static COOKIE: Lazy<Option<Scores>> = Lazy::new(|| {
        let scores = block_on(fetch_scores_storage());
        scores
    });

    COOKIE.unwrap_or_else(Scores::mid)
}

pub async fn fetch_id_storage() -> Option<Uuid> {
    let eval = eval(
        r#"
        let id = localStorage.getItem('user_id');
        if (id) {
            dioxus.send(id);
        } else {
            dioxus.send(null);
        }
        "#,
    )
    .recv()
    .await;

    log_to_console(&eval);

    let mut id = eval.ok()?.to_string();
    id.remove(0);
    id.pop();
    let uuid = Uuid::from_str(&id);
    log_to_console(&uuid);
    uuid.ok()
}

pub async fn fetch_scores_storage() -> Option<Scores> {
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

pub fn test_msg() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            font_size: "0.8em",
            align_items: "center",
            color: "#666",
            div {
                "Unsure? Take the "
                Link {
                    to: Route::Test {},
                    "test"
                }
                "."
            }
        }
    }
}

pub fn footer() -> Element {
    rsx! {
        div {
       //     margin_top: "20px",
            height: "20px",
            display: "flex",
            flex_direction: "row",
            font_size: "0.8em",
            justify_items: "center",
            color: "#666",
            div {
                Link {
                    to: Route::Privacypolicy {},
                    "Privacy Policy"
                }
            }
        }
    }
}

pub fn manual_msg() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            font_size: "0.8em",
            align_items: "center",
            color: "#666",
            div {
                "Already know your score? Enter them "
                Link {
                    to: Route::Manual {},
                    " manually."
                }
            }
        }
    }
}

pub fn score_cmp(mine: Scores, peer: Scores) -> Element {
    let o_diff = (mine.o - peer.o) as i32;
    let c_diff = (mine.c - peer.c) as i32;
    let e_diff = (mine.e - peer.e) as i32;
    let a_diff = (mine.a - peer.a) as i32;
    let n_diff = (mine.n - peer.n) as i32;

    let ostr = format!(
        "{}% {} openness. {}->{}",
        o_diff.abs(),
        if o_diff < 0 { "higher" } else { "lower" },
        mine.o as u32,
        peer.o as u32
    );
    let cstr = format!(
        "{}% {} conscientiousness. {}->{}",
        c_diff.abs(),
        if c_diff < 0 { "higher" } else { "lower" },
        mine.c,
        peer.c
    );
    let estr = format!(
        "{}% {} extroversion. {}->{}",
        e_diff.abs(),
        if e_diff < 0 { "higher" } else { "lower" },
        mine.e,
        peer.e
    );
    let astr = format!(
        "{}% {} agreeableness. {}->{}",
        a_diff.abs(),
        if a_diff < 0 { "higher" } else { "lower" },
        mine.a,
        peer.a
    );
    let nstr = format!(
        "{}% {} neuroticism. {}->{}",
        n_diff.abs(),
        if n_diff < 0 { "higher" } else { "lower" },
        mine.n,
        peer.n
    );

    let font_size = "1em";

    rsx! {
        div {
            p {font_size: "{font_size}", "{ostr}"}
            p {font_size: "{font_size}", "{cstr}"}
            p {font_size: "{font_size}", "{estr}"}
            p {font_size: "{font_size}", "{astr}"}
            p {font_size: "{font_size}", "{nstr}"}
        }
    }
}

pub fn markdown_converter(s: &str) -> Element {
    let lines: Vec<&str> = s.split("\n").collect();

    rsx! {
        for line in lines {
            if line.starts_with("# ") {
                h1 {
                    "{line[2..]}"
                }
            } else if line.starts_with("## ") {
                h2 {
                    "{line[3..]}"
                }
            } else if line.starts_with("### ") {
                h3 {
                    "{line[4..]}"
                }
            } else {
                p {
                    "{line}"
                }
            }
        }
    }
}

pub async fn connect_to_peer(
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

            use SocketMessage as SM;
            let message = match serde_json::from_str(&txt).unwrap() {
                SM::User(msg) => Message::new(Origin::Peer, msg),
                SM::Info(msg) => Message::new(Origin::Info, msg),
                SM::Ping => {
                    let msg = SM::pong();
                    state.send_message(msg);
                    return;
                }
                SM::Pong => {
                    log_to_console("unexpected pong!");
                    return;
                }
                SM::ConnectionClosed => {
                    log_to_console("received 'connection closed' from server");
                    state.clear_socket();
                    return;
                }
                SM::PeerScores(peer_scores) => {
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
pub enum Origin {
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
    pub origin: Origin,
    pub content: String,
}

impl Message {
    pub fn new(origin: Origin, content: String) -> Self {
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
pub struct MessageListProps {
    messages: Vec<Message>,
}

pub fn MessageList(mut msgs: MessageListProps) -> Element {
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
