use std::fmt;

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
    pub question: &'static str,
    pub trait_: Trait,
    pub flipped: bool,
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
