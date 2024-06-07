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
    pub fn summary(&self) -> &'static str {
        match self {
            Self::Rcoai => include_str!("../../files/sloan_desc/rcoai"),
            Self::Rcoan => include_str!("../../files/sloan_desc/rcoan"),
            Self::Rloan => include_str!("../../files/sloan_desc/rloan"),
            Self::Rloai => include_str!("../../files/sloan_desc/rloai"),
            Self::Rluai => include_str!("../../files/sloan_desc/rluai"),
            Self::Rluan => include_str!("../../files/sloan_desc/rluan"),
            Self::Rcuan => include_str!("../../files/sloan_desc/rcuan"),
            Self::Rcuai => include_str!("../../files/sloan_desc/rcuai"),
            Self::Rcoei => include_str!("../../files/sloan_desc/rcoei"),
            Self::Rcoen => include_str!("../../files/sloan_desc/rcoen"),
            Self::Rloen => include_str!("../../files/sloan_desc/rloen"),
            Self::Rloei => include_str!("../../files/sloan_desc/rloei"),
            Self::Rluei => include_str!("../../files/sloan_desc/rluei"),
            Self::Rluen => include_str!("../../files/sloan_desc/rluen"),
            Self::Rcuen => include_str!("../../files/sloan_desc/rcuen"),
            Self::Rcuei => include_str!("../../files/sloan_desc/rcuei"),
            Self::Scoai => include_str!("../../files/sloan_desc/scoai"),
            Self::Scoan => include_str!("../../files/sloan_desc/scoan"),
            Self::Sloan => include_str!("../../files/sloan_desc/sloan"),
            Self::Sloai => include_str!("../../files/sloan_desc/sloai"),
            Self::Sluai => include_str!("../../files/sloan_desc/sluai"),
            Self::Sluan => include_str!("../../files/sloan_desc/sluan"),
            Self::Scuan => include_str!("../../files/sloan_desc/scuan"),
            Self::Scuai => include_str!("../../files/sloan_desc/scuai"),
            Self::Scoei => include_str!("../../files/sloan_desc/scoei"),
            Self::Scoen => include_str!("../../files/sloan_desc/scoen"),
            Self::Sloen => include_str!("../../files/sloan_desc/sloen"),
            Self::Sloei => include_str!("../../files/sloan_desc/sloei"),
            Self::Sluei => include_str!("../../files/sloan_desc/sluei"),
            Self::Sluen => include_str!("../../files/sloan_desc/sluen"),
            Self::Scuen => include_str!("../../files/sloan_desc/scuen"),
            Self::Scuei => include_str!("../../files/sloan_desc/scuei"),
        }
    }

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
