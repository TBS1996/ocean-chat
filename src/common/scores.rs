use crate::common;
use common::Answer;
use common::Question;
use common::Trait;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;
use strum::IntoEnumIterator;

//#[cfg(not(feature = "server"))]
pub static SCORES: Lazy<Vec<Scores>> = Lazy::new(|| {
    let s = include_str!("../../files/scores");
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

    pub fn mid() -> Self {
        Self {
            o: 50.,
            c: 50.,
            e: 50.,
            a: 50.,
            n: 50.,
        }
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
        let s = s
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(s);

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

#[derive(Debug, Default, Clone, Copy)]
pub struct ScoreTally {
    pub o: u32,
    pub c: u32,
    pub e: u32,
    pub a: u32,
    pub n: u32,
}

impl ScoreTally {
    pub fn load() -> Vec<Self> {
        let s = include_str!("../../files/data.csv");
        let s = &s[..s.len() - 1];

        let mut output = vec![];
        let mut rows = s.split("\n").into_iter();

        let column_names: Vec<&str> = rows.next().unwrap().split("\t").collect();

        for row in rows {
            let columns: Vec<&str> = row.split("\t").collect();
            let mut tally = ScoreTally::default();

            for (idx, column) in columns.iter().enumerate() {
                let column = column.trim_end_matches('\r');
                let column_name = column_names[idx].trim_end_matches('\r');
                if let Ok(question) = Question::from_str(column_name) {
                    let answer_val: u32 = column.parse().unwrap();
                    let answer = Answer::from_val(answer_val);
                    tally.add_answer(question, answer);
                }
            }

            output.push(tally);
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

impl ScoreTally {
    pub fn add_answer(&mut self, question: Question, answer: Answer) {
        let points = if question.is_flipped() {
            6 - answer.into_points()
        } else {
            answer.into_points()
        };

        match question.trait_() {
            Trait::Open => self.o += points,
            Trait::Con => self.c += points,
            Trait::Extro => self.e += points,
            Trait::Agree => self.a += points,
            Trait::Neurotic => self.n += points,
        }
    }
}
