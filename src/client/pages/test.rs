#![allow(non_snake_case)]

use crate::client;
use crate::common;

use client::save_scores;
use client::Route;
use client::State;
use common::Answer;
use common::Question;
use common::ScoreTally;
use common::DISTS;
use common::SHORT_DISTS;
use components::nav_bar::top_bar;
use components::nav_bar::Navbar;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

const Q_QTY: usize = 5 * common::Q_PER_TRAIT;

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

    pub fn reset_short(&self) {
        self.inner.lock().unwrap().reset_short();
    }

    pub fn reset_long(&self) {
        self.inner.lock().unwrap().reset_long();
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

    pub fn shortened(&self) -> bool {
        self.inner.lock().unwrap().shortened
    }
}

struct Inner {
    pending_questions: Vec<Question>,
    answered_questions: Vec<(Question, Answer)>,
    current_question: Signal<Question>,
    progress: Signal<u32>,
    shortened: bool,
}

impl Inner {
    fn new() -> Self {
        let mut s = Self {
            pending_questions: vec![],
            answered_questions: vec![],
            current_question: Signal::new(Question::E1), // dummy value
            progress: Signal::new(0),
            shortened: false,
        };

        s.reset();

        s
    }

    fn update_percentage(&mut self) {
        let tot_q = if self.shortened { 20 } else { 50 };
        let p = (((tot_q - self.pending_questions.len()) as f32 / tot_q as f32) * 100.) as u32;
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

    fn reset_short(&mut self) {
        self.pending_questions = Question::all_questions()
            .into_iter()
            .filter(|q| q.short_version())
            .collect();

        self.answered_questions.clear();
        let current_question = self.pending_questions.last().unwrap();
        *self.current_question.write() = *current_question;
        self.shortened = true;
        self.update_percentage();
    }

    fn reset_long(&mut self) {
        self.pending_questions = Question::all_questions();

        self.answered_questions.clear();
        let current_question = self.pending_questions.last().unwrap();
        *self.current_question.write() = *current_question;
        self.shortened = false;
        self.update_percentage();
    }

    fn reset(&mut self) {
        if self.shortened {
            self.reset_short();
        } else {
            self.reset_long();
        }
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
    let quiz2 = quiz.clone();

    rsx! {
        div {
            display: "flex",
            flex_direction: "column",

            if show_navbar { Navbar{active_chat: false} } else {  { top_bar() } }
            div {
                display: "flex",
                justify_content: "center",
                flex_direction: "column",
                align_items: "center",
                class: "navmargin",

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
                                        let scores = if quiz.shortened() {
                                            SHORT_DISTS.convert(tally)
                                        } else {
                                            DISTS.convert(tally)
                                        };

                                        save_scores(scores);
                                        state.set_scores(scores);
                                        navigator.replace(Route::Personality{});
                                        quiz.reset();

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
                            quiz2.reset();
                        },
                        "reset"
                    }

                    button {
                        class: "mybutton",
                        background_color: "black",
                        width: "150px",
                        prevent_default: "onclick",
                        onclick: move |_| {
                            if quiz.shortened(){
                                quiz.reset_long();
                            } else {
                                quiz.reset_short();

                            };
                        },
                        if quiz.shortened() {
                            "Take full version"
                        } else {
                            "Take short version"
                        }
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
}
