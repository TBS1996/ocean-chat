#![allow(non_snake_case)]

use crate::common::Scores;
use crate::common::IP_ADDR;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::RequestMode;
use web_sys::Response;
use web_sys::WebSocket;

#[wasm_bindgen(start)]
pub fn run_app() {
    launch(App);
}

impl Scores {
    fn from_form(form: &FormData) -> Option<Self> {
        let data = form.values();

        let o: f32 = data.get("o")?.as_value().parse().ok()?;
        let c: f32 = data.get("c")?.as_value().parse().ok()?;
        let e: f32 = data.get("e")?.as_value().parse().ok()?;
        let a: f32 = data.get("a")?.as_value().parse().ok()?;
        let n: f32 = data.get("n")?.as_value().parse().ok()?;

        if o < 0. || o > 100. {
            return None;
        }
        if c < 0. || c > 100. {
            return None;
        }
        if e < 0. || e > 100. {
            return None;
        }
        if a < 0. || a > 100. {
            return None;
        }
        if n < 0. || n > 100. {
            return None;
        }

        Some(Self { o, c, e, a, n })
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/invalid")]
    Invalid {},
    #[route("/chat/:id")]
    Chat { id: String },
}

async fn get_paired_user_id(scores: Scores) -> String {
    let scores_json = serde_json::to_string(&scores).unwrap();
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&scores_json)));

    let pair_url = format!("http://{}/pair", IP_ADDR);
    let request = Request::new_with_str_and_init(&pair_url, &opts).unwrap();
    request
        .headers()
        .set("Content-Type", "application/json")
        .unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();
    let resp: Response = resp_value.dyn_into().unwrap();
    let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
    let text = text.as_string().unwrap();

    log_to_console(&text);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    log_to_console(&json.to_string());

    json["peer_id"].as_str().unwrap().to_string()
}

async fn connect_to_peer(id: String) -> Result<WebSocket, String> {
    log_to_console("Starting to connect");
    let url = format!("ws://{}:3000/connect/{}", IP_ADDR, id);

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
fn Chat(id: String) -> Element {
    let mut socket = use_signal(|| None);

    spawn_local(async move {
        let sock = connect_to_peer(id).await.unwrap();
        *socket.write() = Some(sock);
    });

    rsx! {
            form { onsubmit:  move |event| {
                let x = event.data().values().get("msg").unwrap().as_value();
                if let Some(socket) = socket.write().as_mut() {
                    let res = socket.send_with_str(&x);
                    let res = format!("message submitted: {:?}", res);
                    log_to_console(&res);

                }
            },



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
    use_context_provider(|| Signal::new(Option::<Scores>::None));
    rsx! {
        Router::<Route> {}
    }
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

    rsx! {
            form { onsubmit:  move |event| {
                let scores = Scores::from_form(&event.data());
                if let Some(scores) = scores {
                    spawn_local(async move {
                        let other = get_paired_user_id(scores).await;
                        navigator.replace(Route::Chat{id: other});
                    }) ;
                } else {
                    navigator.replace(Route::Invalid {});
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
