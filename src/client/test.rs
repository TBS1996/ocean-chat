#![allow(non_snake_case)]

use crate::client::Route;
use crate::client::State;
use crate::common::Scores;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

/// using statics everywhere because im too dumb to understand dioxus properly
static QUESTIONS: Lazy<Arc<Mutex<Vec<Question>>>> =
    Lazy::new(|| Arc::new(Mutex::new(all_questions())));

fn o_perc(score: u32) -> f32 {
    static MAP: Lazy<Vec<f32>> = Lazy::new(|| {
        let s = include_str!("../../files/o_map");
        serde_json::from_str(&s).unwrap()
    });

    MAP[score as usize - 10]
}
fn c_perc(score: u32) -> f32 {
    static MAP: Lazy<Vec<f32>> = Lazy::new(|| {
        let s = include_str!("../../files/c_map");
        serde_json::from_str(&s).unwrap()
    });

    MAP[score as usize - 10]
}
fn e_perc(score: u32) -> f32 {
    static MAP: Lazy<Vec<f32>> = Lazy::new(|| {
        let s = include_str!("../../files/e_map");
        serde_json::from_str(&s).unwrap()
    });

    MAP[score as usize - 10]
}
fn a_perc(score: u32) -> f32 {
    static MAP: Lazy<Vec<f32>> = Lazy::new(|| {
        let s = include_str!("../../files/a_map");
        serde_json::from_str(&s).unwrap()
    });

    MAP[score as usize - 10]
}
fn n_perc(score: u32) -> f32 {
    static MAP: Lazy<Vec<f32>> = Lazy::new(|| {
        let s = include_str!("../../files/n_map");
        serde_json::from_str(&s).unwrap()
    });

    MAP[score as usize - 10]
}

#[component]
pub fn Test() -> Element {
    let state = use_context::<State>();
    let mut tally = use_signal(ScoreTally::default);
    let mut curr_question = use_signal(|| QUESTIONS.lock().unwrap().last().copied().unwrap());
    let navigator = use_navigator();

    // cant remember last time i wrote a dumber piece of code than this
    let state1 = state.clone();
    let state2 = state.clone();
    let state3 = state.clone();
    let state4 = state.clone();
    let state5 = state.clone();

    rsx! {  div {
            style { { include_str!("../styles.css") } }
            h1 { "Personality Test" }
              div { class: "input-group",
                    "{curr_question}"
              }
              div { class: "buttons",
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let answer = Answer::Disagree;
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, answer);
                            match QUESTIONS.lock().unwrap().last().copied(){
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = tally.write().into_scores();
                                    state1.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }
                        },
                        "Disagree"
                    }
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let answer = Answer::SlightlyDisagree;
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, answer);
                            match QUESTIONS.lock().unwrap().last().copied(){
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = tally.write().into_scores();
                                    state2.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }
                        },
                        "Slightly disagree"
                    }
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let answer = Answer::Neutral;
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, answer);
                            match QUESTIONS.lock().unwrap().last().copied(){
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = tally.write().into_scores();
                                    state3.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }
                        },
                        "Neutral"
                    }
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let answer = Answer::SlightlyAgree;
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, answer);
                            match QUESTIONS.lock().unwrap().last().copied(){
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = tally.write().into_scores();
                                    state4.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }

                        },
                        "Slightly agree"
                    }
                    button {
                        prevent_default: "onclick",
                        onclick: move |_| {
                            let answer = Answer::Agree;
                            let question = QUESTIONS.lock().unwrap().pop().unwrap();
                            tally.write().add_answer(question, answer);
                            match QUESTIONS.lock().unwrap().last().copied(){
                                Some(next_question) => {
                                    *curr_question.write() = next_question;
                                },
                                None => {
                                    let scores = tally.write().into_scores();
                                    state5.set_scores(scores);
                                    navigator.replace(Route::Chat{});
                                },
                            }
                        },
                        "Agree"
                    }


            }
        }

    }
}

#[derive(Default, Clone, Copy)]
struct ScoreTally {
    o: u32,
    c: u32,
    e: u32,
    a: u32,
    n: u32,
}

impl ScoreTally {
    fn add_answer(&mut self, question: Question, answer: Answer) {
        let points = if question.flipped {
            6 - answer.into_points()
        } else {
            answer.into_points()
        };

        match question.trait_ {
            Trait::Open => self.o += points,
            Trait::Con => self.c += points,
            Trait::Extro => self.e += points,
            Trait::Agree => self.a += points,
            Trait::Neurotic => self.n += points,
        }
    }

    fn into_scores(self) -> Scores {
        let mut s = Scores::default();
        s.o = o_perc(self.o);
        s.c = c_perc(self.c);
        s.e = e_perc(self.e);
        s.a = a_perc(self.a);
        s.n = n_perc(self.n);
        s
    }
}

enum Answer {
    Disagree,
    SlightlyDisagree,
    Neutral,
    SlightlyAgree,
    Agree,
}

impl Answer {
    fn into_points(self) -> u32 {
        match self {
            Self::Disagree => 1,
            Self::SlightlyDisagree => 2,
            Self::Neutral => 3,
            Self::SlightlyAgree => 4,
            Self::Agree => 5,
        }
    }
}

#[derive(Clone, Debug, Copy)]
enum Trait {
    Open,
    Con,
    Extro,
    Agree,
    Neurotic,
}

#[derive(Clone, Debug, Copy)]
struct Question {
    question: &'static str,
    trait_: Trait,
    flipped: bool,
}

impl Question {
    fn new(question: &'static str, trait_: Trait, flipped: bool) -> Self {
        Self {
            question,
            trait_,
            flipped,
        }
    }
}

use std::fmt;

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}

fn all_questions() -> Vec<Question> {
    let extraversion = vec![
        (
            "I don't mind being the center of attention.",
            Trait::Extro,
            false,
        ),
        (
            "I talk to a lot of different people at parties.",
            Trait::Extro,
            false,
        ),
        ("I am the life of the party.", Trait::Extro, false),
        ("I start conversations.", Trait::Extro, false),
        ("I feel comfortable around people.", Trait::Extro, false),
        ("I don't talk a lot.", Trait::Extro, true),
        ("I keep in the background.", Trait::Extro, true),
        ("I have little to say.", Trait::Extro, true),
        (
            "I don't like to draw attention to myself.",
            Trait::Extro,
            true,
        ),
        ("I am quiet around strangers.", Trait::Extro, true),
    ];

    let neuroticism = vec![
        ("I get stressed out easily.", Trait::Neurotic, false),
        ("I worry about things.", Trait::Neurotic, false),
        ("I am easily disturbed.", Trait::Neurotic, false),
        ("I get upset easily.", Trait::Neurotic, false),
        ("I change my mood a lot.", Trait::Neurotic, false),
        ("I have frequent mood swings.", Trait::Neurotic, false),
        ("I get irritated easily.", Trait::Neurotic, false),
        ("I often feel blue.", Trait::Neurotic, false),
        ("I am relaxed most of the time.", Trait::Neurotic, true),
        ("I seldom feel blue.", Trait::Neurotic, true),
    ];

    let agreeableness = vec![
        ("I am interested in people.", Trait::Agree, false),
        ("I sympathize with others' feelings.", Trait::Agree, false),
        ("I have a soft heart.", Trait::Agree, false),
        ("I take time out for others.", Trait::Agree, false),
        ("I feel others' emotions.", Trait::Agree, false),
        ("I make people feel at ease.", Trait::Agree, false),
        ("I feel little concern for others.", Trait::Agree, true),
        ("I am not really interested in others.", Trait::Agree, true),
        ("I insult people.", Trait::Agree, true),
        (
            "I am not interested in other people's problems.",
            Trait::Agree,
            true,
        ),
    ];

    let conscientiousness = vec![
        ("I am always prepared.", Trait::Con, false),
        ("I pay attention to details.", Trait::Con, false),
        ("I get chores done right away.", Trait::Con, false),
        ("I like order.", Trait::Con, false),
        ("I follow a schedule.", Trait::Con, false),
        ("I am exacting in my work.", Trait::Con, false),
        ("I leave my belongings around.", Trait::Con, true),
        ("I shirk my duties.", Trait::Con, true),
        ("I make a mess of things.", Trait::Con, true),
        (
            "I often forget to put things back in their proper place.",
            Trait::Con,
            true,
        ),
    ];

    let openness = vec![
        ("I have a rich vocabulary.", Trait::Open, false),
        ("I have a vivid imagination.", Trait::Open, false),
        ("I have excellent ideas.", Trait::Open, false),
        ("I am quick to understand things.", Trait::Open, false),
        ("I use difficult words.", Trait::Open, false),
        ("I spend time reflecting on things.", Trait::Open, false),
        ("I am full of ideas.", Trait::Open, false),
        ("I do not have a good imagination.", Trait::Open, true),
        ("I am not interested in abstract ideas.", Trait::Open, true),
        (
            "I have difficulty understanding abstract ideas.",
            Trait::Open,
            true,
        ),
    ];

    let questions: Vec<Question> = extraversion
        .into_iter()
        .chain(neuroticism)
        .chain(agreeableness)
        .chain(conscientiousness)
        .chain(openness)
        .map(|(question, trait_, flipped)| Question::new(question, trait_, flipped))
        .collect();

    questions
}
