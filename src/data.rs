use crate::common::Scores;

impl Scores {
    fn from_row(row: &[&str]) -> Self {
        let mut f = Scores::default();

        for (idx, col) in row[7..].iter().enumerate() {
            let mut val: u32 = col.trim().parse().unwrap();

            if val == 0 {
                panic!();
            }

            let is_flipped = idx % 2 != 0;

            if is_flipped {
                val = 6 - val;
            }

            if idx < 10 {
                f.e += val;
            } else if idx < 20 {
                f.n += val;
            } else if idx < 30 {
                f.a += val;
            } else if idx < 40 {
                f.c += val;
            } else {
                f.o += val;
            }
        }

        f
    }
}

#[derive(Default, Debug)]
struct Perc {
    o: f32,
    c: f32,
    e: f32,
    a: f32,
    n: f32,
}

impl Perc {
    fn from_points(scores: Scores, o: &X, c: &X, e: &X, a: &X, n: &X) -> Self {
        let mut f = Self::default();
        f.o = *o.get(&scores.o).unwrap();
        f.c = *c.get(&scores.c).unwrap();
        f.e = *e.get(&scores.e).unwrap();
        f.a = *a.get(&scores.a).unwrap();
        f.n = *n.get(&scores.n).unwrap();
        f
    }

    fn euclidean_distance(&self, other: &Perc) -> f32 {
        ((self.o - other.o).powi(2)
            + (self.c - other.c).powi(2)
            + (self.e - other.e).powi(2)
            + (self.a - other.a).powi(2)
            + (self.n - other.n).powi(2))
        .sqrt()
    }

    fn to_string(&self) -> String {
        format!(
            "{}, {}, {}, {}, {}\n",
            self.o, self.c, self.e, self.a, self.n
        )
    }
}

use std::fmt::write;
use std::io::Write;
use std::time::Instant;

type X = HashMap<u32, f32>;

fn main() {
    let mut output = vec![];
    let s = include_str!("data.csv");
    let s = &s[..s.len() - 1];

    let mut x = s.split("\n").into_iter();
    x.next();

    for row in x {
        let cols = row.split("\t");
        let x: Vec<&str> = cols.collect();
        output.push(Scores::from_row(&x));
    }

    // Collect scores for each trait
    let mut e_scores = vec![];
    let mut n_scores = vec![];
    let mut a_scores = vec![];
    let mut c_scores = vec![];
    let mut o_scores = vec![];

    for score in &output {
        e_scores.push(score.e);
        n_scores.push(score.n);
        a_scores.push(score.a);
        c_scores.push(score.c);
        o_scores.push(score.o);
    }

    // Calculate percentiles for each trait
    let e = calculate_percentiles(&e_scores);

    let s: String = serde_json::to_string_pretty(&e.to_vec()).unwrap();
    let mut f = std::fs::File::create("e_map").unwrap();
    f.write_all(s.as_bytes()).unwrap();

    let n = calculate_percentiles(&n_scores);

    let s: String = serde_json::to_string_pretty(&n.to_vec()).unwrap();
    let mut f = std::fs::File::create("n_map").unwrap();
    f.write_all(s.as_bytes()).unwrap();

    let a = calculate_percentiles(&a_scores);

    let s: String = serde_json::to_string_pretty(&a.to_vec()).unwrap();
    let mut f = std::fs::File::create("a_map").unwrap();
    f.write_all(s.as_bytes()).unwrap();

    let c = calculate_percentiles(&c_scores);

    let s: String = serde_json::to_string_pretty(&c.to_vec()).unwrap();
    let mut f = std::fs::File::create("c_map").unwrap();
    f.write_all(s.as_bytes()).unwrap();

    let o = calculate_percentiles(&o_scores);

    let s: String = serde_json::to_string_pretty(&o.to_vec()).unwrap();
    let mut f = std::fs::File::create("o_map").unwrap();
    f.write_all(s.as_bytes()).unwrap();

    /*

    let mut foo = vec![];
    for score in output {
        let f = Perc::from_points(score, &o, &c, &e, &a, &n);
        foo.push(f);
    }

    let mut f = std::fs::File::create_new("scores").unwrap();
    for x in &foo {
        let s = x.to_string();
        f.write_all(s.as_bytes()).unwrap();
    }

    let my_scores = Perc {
        o: 92.,
        c: 0.,
        e: 98.,
        a: 23.,
        n: 5.,
    };

    let mut distances = vec![];

    let start = Instant::now();
    for f in foo {
        let distance: u32 = my_scores.euclidean_distance(&f) as u32;
        distances.push(distance);
    }

    let ff = start.elapsed();
    dbg!(ff);

    let len = distances.len();
    let closer = distances.iter().filter(|d| d < &&100).count();
    let ratio = closer as f32 / len as f32;
    let from_100 = (ratio * 100.) as u32;

    println!(
        "in a room of 100 people, {} have a closer personality than your peer",
        from_100
    );

    */
}

use std::collections::HashMap;

fn calculate_percentiles(scores: &[u32]) -> [f32; 41] {
    let mut raw = raw_percentiles(scores);
    normalize(&mut raw);
    raw
}

fn normalize(scores: &mut [f32; 41]) {
    let mut idx = scores.len() - 1;
    loop {
        if scores[idx] == 0. {
            scores[idx] = 100.;
        } else {
            break;
        }

        idx -= 1;
    }

    idx = scores.iter().position(|num| *num != 0.).unwrap();
    let mut inside = false;
    let mut val_idx = idx;
    let mut val = scores[idx];
    let mut prev_val = val;
    loop {
        let curr_val = scores[idx];

        match (curr_val == 0., inside) {
            (true, true) => {}
            (false, false) => {}
            (true, false) => {
                val = prev_val;
                val_idx = idx - 1;
                inside = true;
            }
            (false, true) => {
                inside = false;
                let steps = idx - val_idx - 1;
                let from = val;
                let to = curr_val;
                let intrapolated = intrapolate(from, to, steps);

                for (i, idx) in (val_idx + 1..idx).into_iter().enumerate() {
                    scores[idx] = intrapolated[i];
                }

                val_idx = idx;
                val = curr_val;
            }
        }
        prev_val = curr_val;

        idx += 1;

        if idx == scores.len() {
            break;
        }
    }
}

fn raw_percentiles(scores: &[u32]) -> [f32; 41] {
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

fn intrapolate(from: f32, to: f32, steps: usize) -> Vec<f32> {
    dbg!(from, to, steps);
    let mut output = vec![];
    let diff = to - from;
    let interval = diff / (steps + 1) as f32;

    for i in 1..=steps {
        output.push(from + i as f32 * interval);
    }

    output
}
