use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::num::ParseFloatError;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::load()));
#[cfg(not(feature = "server"))]
pub static DISTS: Lazy<Distributions> = Lazy::new(|| Distributions::load());

#[cfg(not(feature = "server"))]
pub static SCORES: Lazy<Vec<Scores>> = Lazy::new(|| {
    let s = include_str!("../files/scores");
    let mut scores = vec![];
    for row in s.split("\n") {
        if row.is_empty() {
            continue;
        }

        let s = Scores::from_str(&row).unwrap();
        scores.push(s);
    }

    scores
});

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub local: bool,
    pub pair_interval_millis: u64,
}

impl Config {
    /// Loads the config file.
    ///
    /// Behaviour is different between backend and frontend.
    /// In the backend we load the config from file, creating new if it doesn't exist.
    ///
    /// The frontend however, needs to package the config into the binary since it doesn't
    /// have access to the backend's file system. We use include_str which will make the program
    /// fail to compile if we haven't configured a config yet. This means in practice that if you
    /// didn't manually add a config file, you need to run the backend at least once before
    /// you build the frontend.
    pub fn load() -> Self {
        #[cfg(not(feature = "server"))]
        let config_str = include_str!("../config.toml");
        #[cfg(feature = "server")]
        let config_str = {
            let config_path = std::path::PathBuf::from("config.toml");
            if !config_path.exists() {
                let s: String = toml::to_string(&Self::default()).unwrap();
                std::fs::write(&config_path, s.as_bytes()).unwrap();
            }
            std::fs::read_to_string(&config_path).unwrap()
        };

        toml::from_str(&config_str).unwrap()
    }

    pub fn server_address(&self) -> &'static str {
        if self.local {
            "ws://127.0.0.1:3000"
        } else {
            "wss://oceanchat.app"
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            local: true,
            pair_interval_millis: 1000,
        }
    }
}

#[cfg(feature = "server")]
use axum::extract::ws::Message;

/// The type that gets sent from server to client through socket.
#[derive(Serialize, Deserialize)]
pub enum SocketMessage {
    User(String),
    Info(String),
    PeerScores(Scores),
}

impl SocketMessage {
    #[cfg(feature = "server")]
    pub fn user_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::User(msg)).unwrap();
        Message::Text(s)
    }

    #[cfg(feature = "server")]
    pub fn info_msg(msg: String) -> Message {
        let s = serde_json::to_string(&Self::Info(msg)).unwrap();
        Message::Text(s)
    }

    #[cfg(feature = "server")]
    pub fn peer_scores(scores: Scores) -> Message {
        let s = serde_json::to_string(&Self::PeerScores(scores)).unwrap();
        Message::Text(s)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct Scores {
    pub o: f32,
    pub c: f32,
    pub e: f32,
    pub a: f32,
    pub n: f32,
}

impl Scores {
    /// Calculates euclidean distance between two scores.
    #[allow(dead_code)]
    pub fn distance(&self, other: &Self) -> f32 {
        let open = self.o - other.o;
        let con = self.c - other.c;
        let extro = self.e - other.e;
        let agreeable = self.a - other.a;
        let neurotic = self.n - other.n;

        let diff_sum =
            open.powi(2) + con.powi(2) + extro.powi(2) + agreeable.powi(2) + neurotic.powi(2);

        diff_sum.sqrt()
    }

    /// Returns the percentage of people who are more similar than the given peer.
    #[cfg(not(feature = "server"))]
    pub fn percentage_similarity(self, other: Scores) -> f32 {
        let distance = self.distance(&other);
        let closer = SCORES
            .iter()
            .filter(|score| score.distance(&self) < distance)
            .count();
        let ratio = closer as f32 / SCORES.len() as f32;
        ratio * 100.
    }
}

impl Display for Scores {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2},{:.2},{:.2},{:.2},{:.2}",
            self.o, self.c, self.e, self.a, self.n
        )
    }
}

impl FromStr for Scores {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(',').collect();

        let o = values[0].trim().parse()?;
        let c = values[1].trim().parse()?;
        let e = values[2].trim().parse()?;
        let a = values[3].trim().parse()?;
        let n = values[4].trim().parse()?;

        Ok(Self { o, c, e, a, n })
    }
}

impl TryFrom<&FormData> for Scores {
    type Error = ();

    fn try_from(form: &FormData) -> Result<Self, Self::Error> {
        let data = form.values();

        let o: f32 = data.get("o").unwrap().as_value().parse().map_err(|_| ())?;
        let c: f32 = data.get("c").unwrap().as_value().parse().map_err(|_| ())?;
        let e: f32 = data.get("e").unwrap().as_value().parse().map_err(|_| ())?;
        let a: f32 = data.get("a").unwrap().as_value().parse().map_err(|_| ())?;
        let n: f32 = data.get("n").unwrap().as_value().parse().map_err(|_| ())?;

        if [o, c, e, a, n]
            .iter()
            .all(|&score| (0.0..=100.0).contains(&score))
        {
            Ok(Scores { o, c, e, a, n })
        } else {
            Err(())
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct ScoreTally {
    pub o: u32,
    pub c: u32,
    pub e: u32,
    pub a: u32,
    pub n: u32,
}

impl ScoreTally {
    fn from_row(row: &[&str]) -> Self {
        let questions = Question::all();
        let mut f = ScoreTally::default();

        for (idx, col) in row[7..].iter().enumerate() {
            let val: u32 = col.trim().parse().unwrap();
            let question = questions[idx];
            let answer = Answer::from_val(val);
            f.add_answer(question, answer);
        }

        f
    }

    pub fn load() -> Vec<Self> {
        let s = include_str!("../files/data.csv");
        let s = &s[..s.len() - 1];

        let mut output = vec![];

        let mut x = s.split("\n").into_iter();
        x.next();

        for row in x {
            let cols = row.split("\t");
            let x: Vec<&str> = cols.collect();
            let score = Self::from_row(&x);
            output.push(score);
        }

        output
    }
}

impl Display for ScoreTally {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.2},{:.2},{:.2},{:.2},{:.2}",
            self.o, self.c, self.e, self.a, self.n
        )
    }
}

impl FromStr for ScoreTally {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(',').collect();

        let o = values[0].trim().parse().unwrap();
        let c = values[1].trim().parse().unwrap();
        let e = values[2].trim().parse().unwrap();
        let a = values[3].trim().parse().unwrap();
        let n = values[4].trim().parse().unwrap();

        Ok(Self { o, c, e, a, n })
    }
}

#[derive(Clone, Copy)]
pub enum Answer {
    Disagree,
    SlightlyDisagree,
    Neutral,
    SlightlyAgree,
    Agree,
}

impl ToString for Answer {
    fn to_string(&self) -> String {
        match self {
            Self::Disagree => "Disagree",
            Self::SlightlyDisagree => "Slightly disagree",
            Self::Neutral => "Neutral",
            Self::SlightlyAgree => "Slightly agree",
            Self::Agree => "Agree",
        }
        .to_string()
    }
}

impl Answer {
    pub const ALL: [Answer; 5] = [
        Answer::Disagree,
        Answer::SlightlyDisagree,
        Answer::Neutral,
        Answer::SlightlyAgree,
        Answer::Agree,
    ];

    pub fn from_val(val: u32) -> Self {
        match val {
            1 => Answer::Disagree,
            2 => Answer::SlightlyDisagree,
            3 => Answer::Neutral,
            4 => Answer::SlightlyAgree,
            5 => Answer::Agree,
            _ => panic!(),
        }
    }

    pub fn into_points(self) -> u32 {
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
pub enum Trait {
    Open,
    Con,
    Extro,
    Agree,
    Neurotic,
}

#[derive(Clone, Debug, Copy)]
pub struct Question {
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

    pub fn all() -> Vec<Self> {
        let extraversion = vec![
            ("I am the life of the party.", Trait::Extro, false),
            ("I don't talk a lot.", Trait::Extro, true),
            ("I feel comfortable around people.", Trait::Extro, false),
            ("I keep in the background.", Trait::Extro, true),
            ("I start conversations.", Trait::Extro, false),
            ("I have little to say.", Trait::Extro, true),
            (
                "I talk to a lot of different people at parties.",
                Trait::Extro,
                false,
            ),
            (
                "I don't like to draw attention to myself.",
                Trait::Extro,
                true,
            ),
            (
                "I don't mind being the center of attention.",
                Trait::Extro,
                false,
            ),
            ("I am quiet around strangers.", Trait::Extro, true),
        ];

        let neuroticism = vec![
            ("I get stressed out easily.", Trait::Neurotic, false),
            ("I am relaxed most of the time.", Trait::Neurotic, true),
            ("I worry about things.", Trait::Neurotic, false),
            ("I seldom feel blue.", Trait::Neurotic, true),
            ("I am easily disturbed.", Trait::Neurotic, false),
            ("I get upset easily.", Trait::Neurotic, false),
            ("I change my mood a lot.", Trait::Neurotic, false),
            ("I have frequent mood swings.", Trait::Neurotic, false),
            ("I get irritated easily.", Trait::Neurotic, false),
            ("I often feel blue.", Trait::Neurotic, false),
        ];

        let agreeableness = vec![
            ("I feel little concern for others.", Trait::Agree, true),
            ("I am interested in people.", Trait::Agree, false),
            ("I insult people.", Trait::Agree, true),
            ("I sympathize with others' feelings.", Trait::Agree, false),
            (
                "I am not interested in other people's problems.",
                Trait::Agree,
                true,
            ),
            ("I have a soft heart.", Trait::Agree, false),
            ("I am not really interested in others.", Trait::Agree, true),
            ("I take time out for others.", Trait::Agree, false),
            ("I feel others' emotions.", Trait::Agree, false),
            ("I make people feel at ease.", Trait::Agree, false),
        ];

        let conscientiousness = vec![
            ("I am always prepared.", Trait::Con, false),
            ("I leave my belongings around.", Trait::Con, true),
            ("I pay attention to details.", Trait::Con, false),
            ("I make a mess of things.", Trait::Con, true),
            ("I get chores done right away.", Trait::Con, false),
            (
                "I often forget to put things back in their proper place.",
                Trait::Con,
                true,
            ),
            ("I like order.", Trait::Con, false),
            ("I shirk my duties.", Trait::Con, true),
            ("I follow a schedule.", Trait::Con, false),
            ("I am exacting in my work.", Trait::Con, false),
        ];

        let openness = vec![
            ("I have a rich vocabulary.", Trait::Open, false),
            (
                "I have difficulty understanding abstract ideas.",
                Trait::Open,
                true,
            ),
            ("I have a vivid imagination.", Trait::Open, false),
            ("I am not interested in abstract ideas.", Trait::Open, true),
            ("I have excellent ideas.", Trait::Open, false),
            ("I do not have a good imagination.", Trait::Open, true),
            ("I am quick to understand things.", Trait::Open, false),
            ("I use difficult words.", Trait::Open, false),
            ("I spend time reflecting on things.", Trait::Open, false),
            ("I am full of ideas.", Trait::Open, false),
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
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.question)
    }
}

impl ScoreTally {
    pub fn add_answer(&mut self, question: Question, answer: Answer) {
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
}

#[derive(Debug)]
pub struct Distributions {
    pub o: [f32; 41],
    pub c: [f32; 41],
    pub e: [f32; 41],
    pub a: [f32; 41],
    pub n: [f32; 41],
}

impl Distributions {
    #[cfg(not(feature = "server"))]
    fn load() -> Self {
        let s: &'static str = include_str!("../files/dist");
        let mut map: HashMap<String, Vec<f32>> = serde_json::from_str(&s).unwrap();

        Self {
            o: std::mem::take(&mut map.get_mut("o"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            c: std::mem::take(&mut map.get_mut("c"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            e: std::mem::take(&mut map.get_mut("e"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            a: std::mem::take(&mut map.get_mut("a"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            n: std::mem::take(&mut map.get_mut("n"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
        }
    }

    pub fn write_to_disk(&self) {
        let mut map_repr: HashMap<String, Vec<f32>> = HashMap::new();
        map_repr.insert("o".into(), self.o.to_vec());
        map_repr.insert("c".into(), self.c.to_vec());
        map_repr.insert("e".into(), self.e.to_vec());
        map_repr.insert("a".into(), self.a.to_vec());
        map_repr.insert("n".into(), self.n.to_vec());

        let s: String = serde_json::to_string_pretty(&map_repr).unwrap();
        let p = PathBuf::from("files/dist");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(&s.as_bytes()).unwrap();
    }

    pub fn convert(&self, tally: ScoreTally) -> Scores {
        Scores {
            o: self.o[tally.o as usize - 10],
            c: self.c[tally.c as usize - 10],
            e: self.e[tally.e as usize - 10],
            a: self.a[tally.a as usize - 10],
            n: self.n[tally.n as usize - 10],
        }
    }

    pub fn from_tallies(tallies: &Vec<ScoreTally>) -> Self {
        let mut e_scores = vec![];
        let mut n_scores = vec![];
        let mut a_scores = vec![];
        let mut c_scores = vec![];
        let mut o_scores = vec![];

        for score in tallies {
            o_scores.push(score.o);
            c_scores.push(score.c);
            e_scores.push(score.e);
            a_scores.push(score.a);
            n_scores.push(score.n);
        }

        let o = calculate_percentiles(&o_scores);
        let c = calculate_percentiles(&c_scores);
        let e = calculate_percentiles(&e_scores);
        let a = calculate_percentiles(&a_scores);
        let n = calculate_percentiles(&n_scores);

        Self { o, c, e, a, n }
    }
}

fn calculate_percentiles(scores: &[u32]) -> [f32; 41] {
    let mut sorted_scores = scores.to_vec();
    sorted_scores.sort();

    let mut percentiles = HashMap::new();
    let n = sorted_scores.len() as f32;

    for (i, &score) in sorted_scores.iter().enumerate() {
        let percentile = (i as f32 / n) * 100.0;
        percentiles.insert(score, percentile);
    }

    let mut output = [0.; 41];

    for (idx, value) in percentiles {
        output[idx as usize - 10] = value;
    }

    output
}
