#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::save_scores;
use client::Navbar;
use client::Route;
use client::State;
use common::Answer;
use common::Question;
use common::ScoreTally;
use common::DISTS;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

use super::*;

#[derive(Clone)]
pub struct Quiz {
    inner: Arc<Mutex<Inner>>,
}

impl Quiz {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }

    pub fn reset(&self) {
        self.inner.lock().unwrap().reset();
    }

    pub fn go_back(&self) {
        self.inner.lock().unwrap().go_back();
    }

    pub fn next_question(&self, answer: Answer) -> Option<ScoreTally> {
        self.inner.lock().unwrap().next_question(answer)
    }

    pub fn signals(&self) -> (Signal<u32>, Signal<Question>) {
        let s1 = self.inner.lock().unwrap().progress.clone();
        let s2 = self.inner.lock().unwrap().current_question.clone();

        (s1, s2)
    }
}

struct Inner {
    pending_questions: Vec<Question>,
    answered_questions: Vec<(Question, Answer)>,
    current_question: Signal<Question>,
    progress: Signal<u32>,
}

impl Inner {
    fn new() -> Self {
        let mut s = Self {
            pending_questions: vec![],
            answered_questions: vec![],
            current_question: Signal::new(Question::E1), // dummy value
            progress: Signal::new(0),
        };

        s.reset();

        s
    }

    fn update_percentage(&mut self) {
        let p = (((50 - self.pending_questions.len()) as f32 / 50.) * 100.) as u32;
        *self.progress.write() = p;
    }

    fn go_back(&mut self) {
        if self.answered_questions.is_empty() {
            return;
        }

        let (q, _) = self.answered_questions.pop().unwrap();
        self.pending_questions.push(q);
        *self.current_question.write() = q;
        self.update_percentage();
    }

    fn reset(&mut self) {
        self.pending_questions = Question::all_questions();
        self.answered_questions.clear();
        let current_question = self.pending_questions.last().unwrap();
        *self.current_question.write() = *current_question;
        self.update_percentage();
    }

    fn next_question(&mut self, answer: Answer) -> Option<ScoreTally> {
        let q = self.pending_questions.pop().unwrap();
        self.answered_questions.push((q, answer));

        match self.pending_questions.last() {
            Some(q) => {
                *self.current_question.write() = *q;
                self.update_percentage();
                None
            }
            None => {
                let tally = self.tally_up();
                self.reset();
                Some(tally)
            }
        }
    }

    fn tally_up(&self) -> ScoreTally {
        let mut t = ScoreTally::default();

        for (ques, ans) in &self.answered_questions {
            t.add_answer(*ques, *ans);
        }

        t
    }
}

#[component]
pub fn Test() -> Element {
    let state = use_context::<State>();
    let quiz = use_context::<Quiz>();

    let navigator = use_navigator();
    let (progress, curr_question) = quiz.signals();

    let show_navbar = state.scores().is_some();
    let quiz1 = quiz.clone();

    rsx! {
                if show_navbar { Navbar{active_chat: false} } else {  { top_bar() } }
                div {
                    display: "flex",
                    justify_content: "center",
                    margin: "20px",
                    flex_direction: "column",
                    align_items: "center",

                    div {
                        display: "flex",
                        justify_content: "center",
                        font_size: "1.5em",
                        padding_bottom: "30px", "{curr_question}" }
                    div { class: "buttons",
                        for (answer, (state, quiz)) in Answer::ALL.iter().zip(std::iter::repeat((state.clone(), quiz.clone()))) {
                            button {
                                class: "mybutton",
                                prevent_default: "onclick",
                                onclick: move |_| {
                                    if let Some(tally) = quiz.next_question(*answer) {
                                        let scores = DISTS.convert(tally);
                                        save_scores(scores);
                                        state.set_scores(scores);
                                        navigator.replace(Route::Personality{});

                                    }


                                },
                                "{answer}"
                            }
                        }
                    }


                div {
                    margin_top: "20px",
                    display: "flex",
                    flex_direction: "row",
                    justify_content: "center",

                    button {
                        class: "mybutton",
                        background_color: "black",
                        width: "150px",
                        prevent_default: "onclick",
                        onclick: move |_| {
                            quiz1.go_back();
                        },
                        "previous question"
                    }
                    button {
                        class: "mybutton",
                        background_color: "black",
                        width: "150px",
                        prevent_default: "onclick",
                        onclick: move |_| {
                            quiz.reset();
                        },
                        "reset"
                    }

                }

                }
                div {
                    display: "flex",
                    justify_content: "center",
                    align_items: "center",
                    flex_direction: "column",
                    p {"{progress}%"}
                    div {
                        display: "flex",
                        justify_content: "left",
                        background_color: "#f1f1f1",
                        overflow: "hidden",
                        height: "30px",
                        width: "500px",
                        div {
                            display: "flex",
                            align_items: "left",
                            justify_content: "center",
                            height: "100%",
                            color: "white",
                            border_radius: "25px 0 0 25px",
                            transition: "width 0.5s",
                            width: "{progress}%",
                            background_color: "red",
                        }
                    }
                    { manual_msg() }
            }
    }
}
