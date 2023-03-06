use std::cmp::Ordering;
use std::ops::AddAssign;

/// How we store a chess position evaluation.
/// It can be a score, i.e.
/// -2 indicates a 2 point advantage for black.
/// +2.4 indicates a 2.4 point advantage for white.
/// or a checkmate sequence, which is stronger.
/// Checkmate -2 indicates mate in 2 for black.
/// Checkmate 3 indicates mate in 3 for white.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ChessEval {
    /// Indicate that the evaluation detects a score difference.
    /// positive values in favor of white, negative values in favor of black
    Score(f32),
    /// Indicate that the evaluation detects checkmate sequences.
    /// and the i8 indicate in how many moves.
    /// negative values for black, positive values for white.
    /// Checkmate 0 is not valid.
    Checkmate(i8),
}

impl PartialOrd for ChessEval {
    fn partial_cmp(&self, other: &ChessEval) -> Option<Ordering> {
        match (self, other) {
            (Self::Score(a), Self::Score(b)) => {
                if a > b {
                    Some(Ordering::Greater)
                } else if b > a {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Equal)
                }
            }
            (Self::Score(_), Self::Checkmate(b)) => {
                if b < &0 {
                    Some(Ordering::Greater)
                } else if b > &0 {
                    Some(Ordering::Less)
                } else {
                    // Checkmate in 0 makes no sense.
                    None
                }
            }
            (Self::Checkmate(a), Self::Score(_)) => {
                if a < &0 {
                    Some(Ordering::Less)
                } else if a > &0 {
                    Some(Ordering::Greater)
                } else {
                    // Checkmate in 0 makes no sense.
                    None
                }
            }
            (Self::Checkmate(a), Self::Checkmate(b)) => {
                if a == &0 || b == &0 {
                    None
                } else if a > b {
                    Some(Ordering::Less)
                } else if b > a {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Equal)
                }
            }
        }
    }
}

impl AddAssign for ChessEval {
    fn add_assign(&mut self, other: Self) {
        match (*self, other) {
            // 2 scores, we just add up:
            (Self::Score(a), Self::Score(b)) => *self = Self::Score(a + b),
            (Self::Score(_), Self::Checkmate(b)) => *self = Self::Checkmate(b),
            (Self::Checkmate(a), Self::Score(_)) => *self = Self::Checkmate(a),
            (Self::Checkmate(a), Self::Checkmate(b)) => {
                // Take the checkmate with lowest absolute value.
                if a == 0 || b == 0 {
                    return;
                } else if a.abs() > b.abs() {
                    *self = Self::Checkmate(b);
                } else if b.abs() > a.abs() {
                    *self = Self::Checkmate(a);
                }
            }
        }
    }
}

mod tests {
    #[test]
    fn eval_comparison() {
        use crate::chess::eval::ChessEval;
        // Some eval values, from strongest to weakest (from white perspective)
        let eval_1 = ChessEval::Checkmate(3);
        let eval_2 = ChessEval::Checkmate(10);
        let eval_3 = ChessEval::Score(20.4);
        let eval_4 = ChessEval::Score(10.4);
        let eval_5 = ChessEval::Score(-1.4);
        let eval_6 = ChessEval::Score(-3.0);
        let eval_7 = ChessEval::Score(-50.4);
        let eval_8 = ChessEval::Checkmate(-8);
        let eval_9 = ChessEval::Checkmate(-1);

        assert_eq!(true, eval_1 > eval_2);
        assert_eq!(true, eval_2 > eval_3);
        assert_eq!(true, eval_3 > eval_4);
        assert_eq!(true, eval_4 > eval_5);
        assert_eq!(true, eval_5 > eval_6);
        assert_eq!(true, eval_6 > eval_7);
        assert_eq!(true, eval_7 > eval_8);
        assert_eq!(true, eval_8 > eval_9);
        assert_eq!(false, eval_8 == eval_9);
        assert_eq!(true, eval_9 == eval_9);
    }

    #[test]
    fn eval_combination() {
        use crate::chess::eval::ChessEval;
        // Combine evaluation scores.
        let mut eval = ChessEval::Score(-1.4);
        eval += ChessEval::Score(5.4);
        assert_eq!(ChessEval::Score(4.0), eval);

        eval += ChessEval::Score(-4.0);
        assert_eq!(ChessEval::Score(0.0), eval);

        eval += ChessEval::Checkmate(-5);
        assert_eq!(ChessEval::Checkmate(-5), eval);

        eval += ChessEval::Checkmate(-7);
        assert_eq!(ChessEval::Checkmate(-5), eval);

        eval += ChessEval::Score(5394.4);
        assert_eq!(ChessEval::Checkmate(-5), eval);

        eval += ChessEval::Checkmate(3);
        assert_eq!(ChessEval::Checkmate(3), eval);

        eval += ChessEval::Checkmate(0); // Bogus value
        assert_eq!(ChessEval::Checkmate(3), eval);

        eval += ChessEval::Checkmate(-2);
        assert_eq!(ChessEval::Checkmate(-2), eval);

        eval += ChessEval::Checkmate(1);
        assert_eq!(ChessEval::Checkmate(1), eval);
    }
}
