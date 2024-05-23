struct Scores {
    open: f32,
    con: f32,
    extroverted: f32,
    agreeable: f32,
    neurotic: f32,
}

impl Scores {
    pub fn new(
        open: f32,
        con: f32,
        extroverted: f32,
        agreeable: f32,
        neurotic: f32,
    ) -> Option<Self> {
        if open < 0. || open > 100. {
            return None;
        }
        if con < 0. || con > 100. {
            return None;
        }
        if extroverted < 0. || extroverted > 100. {
            return None;
        }
        if agreeable < 0. || agreeable > 100. {
            return None;
        }
        if neurotic < 0. || neurotic > 100. {
            return None;
        }

        Some(Self {
            open,
            con,
            extroverted,
            agreeable,
            neurotic,
        })
    }

    /// Calculates euclidean distance between two scores.
    fn distance(&self, other: &Self) -> f32 {
        let open = self.open - other.open;
        let con = self.con - other.con;
        let extro = self.extroverted - other.extroverted;
        let agreeable = self.agreeable - other.agreeable;
        let neurotic = self.neurotic - other.neurotic;

        let diff_sum =
            open.powi(2) + con.powi(2) + extro.powi(2) + agreeable.powi(2) + neurotic.powi(2);

        diff_sum.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_same_scores() {
        let scores1 = Scores {
            open: 4.5,
            con: 3.2,
            extroverted: 5.0,
            agreeable: 2.8,
            neurotic: 3.5,
        };

        let scores2 = Scores {
            open: 4.5,
            con: 3.2,
            extroverted: 5.0,
            agreeable: 2.8,
            neurotic: 3.5,
        };

        assert_eq!(scores1.distance(&scores2), 0.0);
    }
}
