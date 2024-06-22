use super::*;

pub fn Pretest() -> Element {
    let navigator = use_navigator();
    let quiz = use_context::<Quiz>();
    let quiz2 = quiz.clone();

    rsx! {
        { top_bar() }
        div {
            display: "flex",
            justify_content: "center",
            flex_direction: "column",
            align_items: "center",
            p {
                margin_top: "50px",
                "Take the test so we can match you with the right person!",
            }
            div {
                class: "narrowcol",

                div {
                    class: "main-box",
                    height: "100px",
                    text_align: "center",
                    line_height: "40px",
                    padding: "10px",
                    margin: "20px",
                    onclick: move |_| {
                        quiz.reset_short();
                        navigator.push(Route::Test{});
                    },
                    h2 { "Short test"} h3 {"1-2 minutes" }
                }
                div {
                    class: "main-box",
                    height: "100px",
                    text_align: "center",
                    line_height: "40px",
                    padding: "10px",
                    margin: "20px",

                    onclick: move |_| {
                        quiz2.reset_long();
                        navigator.push(Route::Test{});
                    },
                    h2 { "Full test" } h3 {"3-4 minutes"}
                }
            }
            { manual_msg() }
        }
    }
}
