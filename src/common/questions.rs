use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

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

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Trait {
    Open,
    Con,
    Extro,
    Agree,
    Neurotic,
}

impl Trait {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Open => "#1E90FF",
            Self::Con => "#32CD32",
            Self::Extro => "#FF4500",
            Self::Agree => "#FFD700",
            Self::Neurotic => "#FF8C00",
        }
    }
}

impl fmt::Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Open => "Openness",
            Self::Con => "Conscientiousness",
            Self::Extro => "Extraversion",
            Self::Agree => "Agreeableness",
            Self::Neurotic => "Neuroticism",
        };

        write!(f, "{}", s)
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

#[derive(Debug, EnumString, EnumIter, Clone, Copy)]
pub enum Question {
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    E9,
    E10,

    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,

    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    A10,

    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,

    O1,
    O2,
    O3,
    O4,
    O5,
    O6,
    O7,
    O8,
    O9,
    O10,
}

impl Question {
    pub fn all_questions() -> Vec<Self> {
        use Question::*;

        vec![
            O1, C1, E1, A1, N1, O2, C2, E2, A2, N2, O3, C3, E3, A3, N3, O4, C4, E4, A4, N4, O5, C5,
            E5, A5, N5, O6, C6, E6, A6, N6, O7, C7, E7, A7, N7, O8, C8, E8, A8, N8, O9, C9, E9, A9,
            N9, O10, C10, E10, A10, N10,
        ]
    }

    pub fn text(&self) -> &'static str {
        match self {
            Question::E1 => "I am the life of the party.",
            Question::E2 => "I don't talk a lot.",
            Question::E3 => "I feel comfortable around people.",
            Question::E4 => "I keep in the background.",
            Question::E5 => "I start conversations.",
            Question::E6 => "I have little to say.",
            Question::E7 => "I talk to a lot of different people at parties.",
            Question::E8 => "I don't like to draw attention to myself.",
            Question::E9 => "I don't mind being the center of attention.",
            Question::E10 => "I am quiet around strangers.",

            Question::N1 => "I get stressed out easily.",
            Question::N2 => "I am relaxed most of the time.",
            Question::N3 => "I worry about things.",
            Question::N4 => "I seldom feel blue.",
            Question::N5 => "I am easily disturbed.",
            Question::N6 => "I get upset easily.",
            Question::N7 => "I change my mood a lot.",
            Question::N8 => "I have frequent mood swings.",
            Question::N9 => "I get irritated easily.",
            Question::N10 => "I often feel blue.",

            Question::A1 => "I feel little concern for others.",
            Question::A2 => "I am interested in people.",
            Question::A3 => "I insult people.",
            Question::A4 => "I sympathize with others' feelings.",
            Question::A5 => "I am not interested in other people's problems.",
            Question::A6 => "I have a soft heart.",
            Question::A7 => "I am not really interested in others.",
            Question::A8 => "I take time out for others.",
            Question::A9 => "I feel others' emotions.",
            Question::A10 => "I make people feel at ease.",

            Question::C1 => "I am always prepared.",
            Question::C2 => "I leave my belongings around.",
            Question::C3 => "I pay attention to details.",
            Question::C4 => "I make a mess of things.",
            Question::C5 => "I get chores done right away.",
            Question::C6 => "I often forget to put things back in their proper place.",
            Question::C7 => "I like order.",
            Question::C8 => "I shirk my duties.",
            Question::C9 => "I follow a schedule.",
            Question::C10 => "I am exacting in my work.",

            Question::O1 => "I have a rich vocabulary.",
            Question::O2 => "I have difficulty understanding abstract ideas.",
            Question::O3 => "I have a vivid imagination.",
            Question::O4 => "I am not interested in abstract ideas.",
            Question::O5 => "I have excellent ideas.",
            Question::O6 => "I do not have a good imagination.",
            Question::O7 => "I am quick to understand things.",
            Question::O8 => "I use difficult words.",
            Question::O9 => "I spend time reflecting on things.",
            Question::O10 => "I am full of ideas.",
        }
    }

    pub fn is_flipped(&self) -> bool {
        match self {
            Question::E1 => false,
            Question::E2 => true,
            Question::E3 => false,
            Question::E4 => true,
            Question::E5 => false,
            Question::E6 => true,
            Question::E7 => false,
            Question::E8 => true,
            Question::E9 => false,
            Question::E10 => true,

            Question::N1 => false,
            Question::N2 => true,
            Question::N3 => false,
            Question::N4 => true,
            Question::N5 => false,
            Question::N6 => false,
            Question::N7 => false,
            Question::N8 => false,
            Question::N9 => false,
            Question::N10 => false,

            Question::A1 => true,
            Question::A2 => false,
            Question::A3 => true,
            Question::A4 => false,
            Question::A5 => true,
            Question::A6 => false,
            Question::A7 => true,
            Question::A8 => false,
            Question::A9 => false,
            Question::A10 => false,

            Question::C1 => false,
            Question::C2 => true,
            Question::C3 => false,
            Question::C4 => true,
            Question::C5 => false,
            Question::C6 => true,
            Question::C7 => false,
            Question::C8 => true,
            Question::C9 => false,
            Question::C10 => false,

            Question::O1 => false,
            Question::O2 => true,
            Question::O3 => false,
            Question::O4 => true,
            Question::O5 => false,
            Question::O6 => true,
            Question::O7 => false,
            Question::O8 => false,
            Question::O9 => false,
            Question::O10 => false,
        }
    }

    pub fn trait_(&self) -> Trait {
        match self {
            Question::E1
            | Question::E2
            | Question::E3
            | Question::E4
            | Question::E5
            | Question::E6
            | Question::E7
            | Question::E8
            | Question::E9
            | Question::E10 => Trait::Extro,
            Question::N1
            | Question::N2
            | Question::N3
            | Question::N4
            | Question::N5
            | Question::N6
            | Question::N7
            | Question::N8
            | Question::N9
            | Question::N10 => Trait::Neurotic,
            Question::A1
            | Question::A2
            | Question::A3
            | Question::A4
            | Question::A5
            | Question::A6
            | Question::A7
            | Question::A8
            | Question::A9
            | Question::A10 => Trait::Agree,
            Question::C1
            | Question::C2
            | Question::C3
            | Question::C4
            | Question::C5
            | Question::C6
            | Question::C7
            | Question::C8
            | Question::C9
            | Question::C10 => Trait::Con,
            Question::O1
            | Question::O2
            | Question::O3
            | Question::O4
            | Question::O5
            | Question::O6
            | Question::O7
            | Question::O8
            | Question::O9
            | Question::O10 => Trait::Open,
        }
    }
}
