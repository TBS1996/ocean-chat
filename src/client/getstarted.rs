use super::*;

pub fn Pretest() -> Element {
    let navigator = use_navigator();

    rsx! {
        { top_bar() }
        div {
            display: "flex",
            justify_content: "center",
            flex_direction: "column",
            align_items: "center",
            p {
                margin_top: "50px",
                "Obviously, to pair you up with similar people we have to know who you are.
    ",
            }
            p {
                "Start taking the standard Big-5 test, and you'll be ready in just 3-5 minutes.",
            }
            div {
                class: "main-box",
                height: "100px",
                line_height: "75px",
                onclick: move |_| {navigator.push(Route::Test{});},
                h2 { "Start test" }
            }
            { manual_msg() }
       //     { footer() }
        }
    }
}
