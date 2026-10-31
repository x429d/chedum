use serde::{Deserialize, Serialize};

use crate::domain::game::Color;

/// Engine score in UCI format: from the perspective of the side to move.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub cp: Option<i32>,
    pub mate: Option<i32>,
}

impl Score {
    pub fn new(cp: Option<i32>, mate: Option<i32>) -> Self {
        if mate.is_some() {
            Score { cp: None, mate }
        } else {
            Score { cp, mate: None }
        }
    }

    pub fn cp_only(cp: i32) -> Self {
        Score {
            cp: Some(cp),
            mate: None,
        }
    }

    pub fn flipped(self) -> Self {
        Score {
            cp: self.cp.map(|value| -value),
            mate: self.mate.map(|value| -value),
        }
    }

    /// Score relative to White.
    pub fn white_relative(self, side_to_move: Color) -> Self {
        match side_to_move {
            Color::White => self,
            Color::Black => self.flipped(),
        }
    }

    /// Win probability of the side to move.
    pub fn win_prob(self) -> f64 {
        match self.mate {
            Some(mate) => win_prob_mate(mate),
            None => win_prob_cp(f64::from(self.cp.unwrap_or(0))),
        }
    }

    /// Win probability of White.
    pub fn win_prob_white(self, side_to_move: Color) -> f64 {
        match side_to_move {
            Color::White => self.win_prob(),
            Color::Black => 100.0 - self.win_prob(),
        }
    }

    /// Numeric value for ACPL: mate is scored as ±1000 centipawns.
    pub fn cp_value(self) -> f64 {
        match self.mate {
            Some(mate) => {
                if mate > 0 {
                    1000.0
                } else {
                    -1000.0
                }
            }
            None => f64::from(self.cp.unwrap_or(0)),
        }
    }

    /// Human-readable score: `+1.45`, `-0.30`, `M3`, `-M2`.
    pub fn display(self) -> String {
        match self.mate {
            Some(mate) if mate > 0 => format!("M{mate}"),
            Some(mate) => format!("-M{}", mate.abs()),
            None => {
                let cp = self.cp.unwrap_or(0);
                let sign = if cp >= 0 { "+" } else { "-" };
                format!("{sign}{:.2}", f64::from(cp.abs()) / 100.0)
            }
        }
    }
}

/// W(cp) = 50 + 50 * (2 / (1 + exp(-0.00368208 * cp)) - 1).
pub fn win_prob_cp(cp: f64) -> f64 {
    let value = 50.0 + 50.0 * (2.0 / (1.0 + (-0.00368208 * cp).exp()) - 1.0);
    value.clamp(0.0, 100.0)
}

/// Mate in N moves: 100 − N*0.2 for the mating side, N*0.2 for the losing side.
pub fn win_prob_mate(mate: i32) -> f64 {
    if mate == 0 {
        return 0.0;
    }
    let distance = f64::from(mate.abs().clamp(0, 20));
    if mate > 0 {
        100.0 - distance * 0.2
    } else {
        distance * 0.2
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn equal_position_is_fifty_percent() {
        assert!((win_prob_cp(0.0) - 50.0).abs() < 1e-9);
    }

    #[test]
    fn positive_cp_favours_side_to_move() {
        assert!(win_prob_cp(100.0) > 50.0);
        assert!(win_prob_cp(-100.0) < 50.0);
        assert!((win_prob_cp(100.0) + win_prob_cp(-100.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn mate_win_probability() {
        assert!((win_prob_mate(3) - 99.4).abs() < 1e-9);
        assert!((win_prob_mate(-3) - 0.6).abs() < 1e-9);
        assert!((win_prob_mate(40) - 96.0).abs() < 1e-9);
        assert!((win_prob_mate(0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn white_relative_flips_for_black() {
        let score = Score::cp_only(120);
        let white = score.white_relative(Color::Black);
        assert_eq!(white.cp, Some(-120));
        assert!(score.win_prob_white(Color::Black) < 50.0);
        assert!(score.win_prob_white(Color::White) > 50.0);
    }

    #[test]
    fn score_display() {
        assert_eq!(Score::cp_only(145).display(), "+1.45");
        assert_eq!(Score::cp_only(-30).display(), "-0.30");
        assert_eq!(Score::new(None, Some(3)).display(), "M3");
        assert_eq!(Score::new(None, Some(-2)).display(), "-M2");
    }
}
