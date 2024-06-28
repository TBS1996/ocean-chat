use super::*;

// #[component]
pub fn top_bar() -> Element {
    rsx! {
        div {
            background_color: "#0a5f98",
            width: "100%",
            margin: "0",
            padding: "20px 0",
            height: "100px",
            display: "flex",
            align_items: "center",
            justify_content: "center",

            div {
                display: "flex",
                align_items: "center",
                justify_content: "center",

                Link {
                    to: Route::Home {},
                    img {
                        src: "logo.svg",
                        alt: "Oceanchat Logo",
                        width: "80px",
                        height: "80px",
                        margin_right: "20px",
                    }
                }

                div {
                    font_size: "2.5em",
                    color: "white",
                    margin: "0",
                    "OceanChat"
                }
            }
        }
    }
}

#[component]
pub fn Navbar(active_chat: bool) -> Element {
    rsx! {
        nav {
            position: "fixed",
            z_index: "1000",

            ul {
                Link {

                    to: Route::Home {},
                    img {
                        src: "logo.svg",
                        alt: "Oceanchat Logo",
                        class: "logo",
                    }

                    background_color: "transparent",
                }
                li {
                    Link {
                        to: Route::Chat {},
                        "Chat",
                        class: if active_chat { "active" } else { "" }
                    }
                }
                li {
                    Link {
                        to: Route::Personality {},
                        "My personality",
                        class: if !active_chat { "active" } else { "" }
                    }
                }
            }
        }
    }
}
