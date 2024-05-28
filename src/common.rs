use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
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
}

impl Display for Scores {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.o, self.c, self.e, self.a, self.n)
    }
}

impl FromStr for Scores {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values: Vec<&str> = s.split(',').collect();
        if values.len() != 5 {}

        let o = values[0].parse()?;
        let c = values[1].parse()?;
        let e = values[2].parse()?;
        let a = values[3].parse()?;
        let n = values[4].parse()?;

        Ok(Self { o, c, e, a, n })
    }
}
