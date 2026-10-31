use crate::domain::analysis::{MoveCategory, MoveEvaluation};
use crate::domain::game::Color;
use crate::domain::position::side_to_move;

/// Maximum number of key moments shown per game.
pub const KEY_MOMENT_MAX: usize = 8;
/// Minimum Win% loss for a move to count as a key moment.
pub const KEY_MOMENT_MIN_LOSS: f64 = 8.0;
/// Minimum distance between key moments, in plies.
pub const KEY_MOMENT_MIN_GAP: u32 = 3;
/// Priority bonus when the position was equal or winning before the move.
pub const KEY_MOMENT_EQUAL_BONUS: f64 = 10.0;
/// Threshold for "was not worse", in Win%.
pub const KEY_MOMENT_EQUAL_WIN: f64 = 50.0;

#[derive(Debug, Clone, Copy)]
pub struct MomentCandidate {
    pub index: usize,
    pub ply: u32,
    pub loss: f64,
    pub was_equal_or_better: bool,
}

impl MomentCandidate {
    fn priority(&self) -> f64 {
        if self.was_equal_or_better {
            self.loss + KEY_MOMENT_EQUAL_BONUS
        } else {
            self.loss
        }
    }
}

/// A significant Win% loss outranks the category label: an inaccuracy at or
/// above the threshold also becomes a key moment.
fn is_error_category(category: MoveCategory) -> bool {
    matches!(
        category,
        MoveCategory::Inaccuracy
            | MoveCategory::Miss
            | MoveCategory::Mistake
            | MoveCategory::Blunder
    )
}

/// Selects the player's key moments: inaccuracies, mistakes, blunders, and
/// missed chances with a loss of at least the threshold, no closer than
/// `KEY_MOMENT_MIN_GAP` plies to each other. Returns the indices of the selected
/// moves in the original array.
pub fn select_key_moments(
    moves: &[MoveEvaluation],
    user_color: Option<Color>,
    player_of: impl Fn(&MoveEvaluation) -> Option<Color>,
) -> Vec<usize> {
    let Some(user_color) = user_color else {
        return Vec::new();
    };
    let mut candidates: Vec<MomentCandidate> = Vec::new();
    for (index, mv) in moves.iter().enumerate() {
        if player_of(mv) != Some(user_color) {
            continue;
        }
        if !is_error_category(mv.classification) || mv.win_prob_loss < KEY_MOMENT_MIN_LOSS {
            continue;
        }
        let best_win = mv
            .pv_lines
            .first()
            .map(|line| line.win_prob)
            .unwrap_or(KEY_MOMENT_EQUAL_WIN);
        candidates.push(MomentCandidate {
            index,
            ply: mv.ply.max(1) as u32,
            loss: mv.win_prob_loss,
            was_equal_or_better: best_win >= KEY_MOMENT_EQUAL_WIN,
        });
    }
    candidates.sort_by(|left, right| {
        right
            .priority()
            .total_cmp(&left.priority())
            .then_with(|| left.ply.cmp(&right.ply))
    });
    let mut chosen: Vec<MomentCandidate> = Vec::new();
    for candidate in candidates {
        if chosen.len() >= KEY_MOMENT_MAX {
            break;
        }
        let too_close = chosen
            .iter()
            .any(|other| other.ply.abs_diff(candidate.ply) < KEY_MOMENT_MIN_GAP);
        if !too_close {
            chosen.push(candidate);
        }
    }
    chosen.sort_by_key(|candidate| candidate.ply);
    chosen
        .into_iter()
        .map(|candidate| candidate.index)
        .collect()
}

/// Side that made the move: more reliable than ply parity, and also works for FEN games.
pub fn player_from_fen(mv: &MoveEvaluation) -> Option<Color> {
    crate::domain::position::parse_fen(&mv.fen_before)
        .ok()
        .map(|pos| side_to_move(&pos))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::analysis::{Phase, PvLine, TimeFlag};

    fn mv(ply: i32, loss: f64, category: MoveCategory, best_win: f64) -> MoveEvaluation {
        MoveEvaluation {
            game_id: 1,
            ply,
            player_color: if ply % 2 == 1 {
                Color::White
            } else {
                Color::Black
            },
            phase: Phase::Middlegame,
            fen_before: "not a fen".to_string(),
            fen_after: "not a fen".to_string(),
            played_uci: "e2e4".to_string(),
            played_san: "e4".to_string(),
            best_uci: None,
            best_san: None,
            eval_cp: None,
            eval_mate: None,
            best_eval_cp: None,
            best_eval_mate: None,
            win_prob_white: 50.0,
            win_prob_loss: loss,
            move_accuracy: 50.0,
            classification: category,
            is_critical: false,
            is_key_moment: false,
            material_diff_white: 0,
            clock_remaining_sec: None,
            time_spent_sec: None,
            time_flag: TimeFlag::Normal,
            explanation_tag: None,
            explanation_params: None,
            explanation_text: None,
            pv_lines: vec![PvLine {
                rank: 1,
                score_cp: None,
                score_mate: None,
                win_prob: best_win,
                uci_moves: Vec::new(),
                san_moves: Vec::new(),
            }],
            user_comment: String::new(),
        }
    }

    fn white_moves(moves: &[MoveEvaluation]) -> Vec<usize> {
        select_key_moments(moves, Some(Color::White), |mv| {
            if mv.ply % 2 == 1 {
                Some(Color::White)
            } else {
                Some(Color::Black)
            }
        })
    }

    #[test]
    fn clean_game_has_no_moments() {
        let moves: Vec<_> = (1..=20)
            .map(|ply| mv(ply, 0.5, MoveCategory::Best, 60.0))
            .collect();
        assert!(white_moves(&moves).is_empty());
    }

    #[test]
    fn ten_blunders_are_capped_at_eight() {
        let moves: Vec<_> = (1..=40)
            .map(|ply| {
                if ply % 4 == 1 {
                    mv(ply, 25.0, MoveCategory::Blunder, 70.0)
                } else {
                    mv(ply, 0.0, MoveCategory::Best, 50.0)
                }
            })
            .collect();
        assert_eq!(white_moves(&moves).len(), KEY_MOMENT_MAX);
    }

    #[test]
    fn selected_moments_are_at_least_three_plies_apart() {
        let moves: Vec<_> = (1..=20)
            .map(|ply| {
                if ply % 2 == 1 && ply <= 9 {
                    mv(ply, 20.0, MoveCategory::Blunder, 70.0)
                } else {
                    mv(ply, 0.0, MoveCategory::Best, 50.0)
                }
            })
            .collect();
        let chosen = white_moves(&moves);
        assert!(!chosen.is_empty());
        for pair in chosen.windows(2) {
            let left = moves[pair[0]].ply;
            let right = moves[pair[1]].ply;
            assert!(right - left >= 3, "{left} -> {right}");
        }
    }

    #[test]
    fn opponent_moves_are_never_selected() {
        let moves: Vec<_> = (1..=20)
            .map(|ply| {
                if ply % 2 == 0 {
                    mv(ply, 30.0, MoveCategory::Blunder, 20.0)
                } else {
                    mv(ply, 0.0, MoveCategory::Best, 50.0)
                }
            })
            .collect();
        assert!(white_moves(&moves).is_empty());
    }

    #[test]
    fn equal_or_winning_positions_get_priority() {
        let moves = vec![
            mv(1, 12.0, MoveCategory::Mistake, 60.0),
            mv(3, 12.0, MoveCategory::Mistake, 20.0),
        ];
        let chosen = white_moves(&moves);
        assert_eq!(chosen, vec![0]);
    }

    #[test]
    fn small_losses_below_threshold_are_ignored() {
        let moves = vec![mv(1, 3.0, MoveCategory::Mistake, 70.0)];
        assert!(white_moves(&moves).is_empty());
    }
}
