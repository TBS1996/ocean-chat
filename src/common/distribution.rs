use crate::common;
use common::Answer;
use common::ScoreTally;
use common::Scores;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Write;
use std::mem;
use std::path::PathBuf;

pub const Q_PER_TRAIT: usize = 4;
pub const CARDINALITY: usize = Q_PER_TRAIT * Answer::MAX_POINTS - Q_PER_TRAIT + 1;

#[cfg(not(feature = "server"))]
pub static DISTS: Lazy<Distributions> = Lazy::new(|| Distributions::load());

#[derive(Debug)]
pub struct Distributions {
    pub o: [f32; CARDINALITY],
    pub c: [f32; CARDINALITY],
    pub e: [f32; CARDINALITY],
    pub a: [f32; CARDINALITY],
    pub n: [f32; CARDINALITY],
}

impl Distributions {
    #[cfg(not(feature = "server"))]
    pub fn load() -> Self {
        let s: &'static str = include_str!("../../files/dist");
        let mut map: HashMap<String, Vec<f32>> = serde_json::from_str(&s).unwrap();

        Self {
            o: mem::take(&mut map.get_mut("o"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            c: mem::take(&mut map.get_mut("c"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            e: mem::take(&mut map.get_mut("e"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            a: mem::take(&mut map.get_mut("a"))
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap(),
            n: mem::take(&mut map.get_mut("n"))
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
        let p = PathBuf::from("files/smalldist");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(&s.as_bytes()).unwrap();
    }

    pub fn convert(&self, tally: ScoreTally) -> Scores {
        Scores {
            o: self.o[tally.o as usize - Q_PER_TRAIT],
            c: self.c[tally.c as usize - Q_PER_TRAIT],
            e: self.e[tally.e as usize - Q_PER_TRAIT],
            a: self.a[tally.a as usize - Q_PER_TRAIT],
            n: self.n[tally.n as usize - Q_PER_TRAIT],
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

fn calculate_percentiles(scores: &[u32]) -> [f32; CARDINALITY] {
    let mut sorted_scores = scores.to_vec();
    sorted_scores.sort();

    let mut percentiles = HashMap::new();
    let n = sorted_scores.len() as f32;

    for (i, &score) in sorted_scores.iter().enumerate() {
        let percentile = (i as f32 / n) * 100.0;
        percentiles.insert(score, percentile);
    }

    let mut output = [0.; CARDINALITY];

    for (idx, value) in percentiles {
        output[idx as usize - Q_PER_TRAIT] = value;
    }

    output
}
