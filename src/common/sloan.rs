use super::*;

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Sloan {
    Rcoai,
    Rcoan,
    Rloan,
    Rloai,
    Rluai,
    Rluan,
    Rcuan,
    Rcuai,
    Rcoei,
    Rcoen,
    Rloen,
    Rloei,
    Rluei,
    Rluen,
    Rcuen,
    Rcuei,
    Scoai,
    Scoan,
    Sloan,
    Sloai,
    Sluai,
    Sluan,
    Scuan,
    Scuai,
    Scoei,
    Scoen,
    Sloen,
    Sloei,
    Sluei,
    Sluen,
    Scuen,
    Scuei,
}

impl Display for Sloan {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Sloan {
    pub fn from_scores(scores: Scores) -> Self {
        let s = scores.e > 50.0;
        let l = scores.n > 50.0;
        let o = scores.c > 50.0;
        let a = scores.a > 50.0;
        let i = scores.o > 50.0;

        match (s, l, o, a, i) {
            (false, false, false, false, false) => Self::Rcuen,
            (false, false, false, false, true) => Self::Rcuei,
            (false, false, false, true, false) => Self::Rcuan,
            (false, false, false, true, true) => Self::Rcuai,
            (false, false, true, false, false) => Self::Rcoen,
            (false, false, true, false, true) => Self::Rcoei,
            (false, false, true, true, false) => Self::Rcoan,
            (false, false, true, true, true) => Self::Rcoai,
            (false, true, false, false, false) => Self::Rluen,
            (false, true, false, false, true) => Self::Rluei,
            (false, true, false, true, false) => Self::Rluan,
            (false, true, false, true, true) => Self::Rluai,
            (false, true, true, false, false) => Self::Rloen,
            (false, true, true, false, true) => Self::Rloei,
            (false, true, true, true, false) => Self::Rloan,
            (false, true, true, true, true) => Self::Rloai,
            (true, false, false, false, false) => Self::Scuen,
            (true, false, false, false, true) => Self::Scuei,
            (true, false, false, true, false) => Self::Scuan,
            (true, false, false, true, true) => Self::Scuai,
            (true, false, true, false, false) => Self::Scoen,
            (true, false, true, false, true) => Self::Scoei,
            (true, false, true, true, false) => Self::Scoan,
            (true, false, true, true, true) => Self::Scoai,
            (true, true, false, false, false) => Self::Sluen,
            (true, true, false, false, true) => Self::Sluei,
            (true, true, false, true, false) => Self::Sluan,
            (true, true, false, true, true) => Self::Sluai,
            (true, true, true, false, false) => Self::Sloen,
            (true, true, true, false, true) => Self::Sloei,
            (true, true, true, true, false) => Self::Sloan,
            (true, true, true, true, true) => Self::Sloai,
        }
    }
}
