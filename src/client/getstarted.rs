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
                "Before you can get started, we must know your big 5 scores",
            }
            p {
                "Take our test, where you answer 50 questions. Estimated 3-5 minutes to complete.",
            }
            div {
                class: "main-box",
                onclick: move |_| {navigator.push(Route::Test{});},
                h2 { "Start test" }
            }
            { manual_msg() }
        }
    }
}
