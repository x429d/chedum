use crate::domain::analysis::MoveCategory;

/// All data needed for priority-based classification of a move.
#[derive(Debug, Clone, Copy)]
pub struct ClassifyInput {
    pub ply: u32,
    pub in_book: bool,
    pub legal_moves: usize,
    /// Loss of win probability on the ply, in percentage points.
    pub delta_w: f64,
    pub played_is_best: bool,
    /// Player's Win% before the move (MultiPV 1 line).
    pub best_win: f64,
    /// Player's Win% after the move actually played.
    pub played_win: f64,
    /// W(pv1) − W(pv2) in percentage points.
    pub second_gap: f64,
    pub sacrifice: bool,
    pub prev_opponent_blunder: bool,
    pub missed_mate: bool,
    pub allowed_mate: bool,
}

/// Priority-ordered tree of 11 categories.
pub fn classify(input: &ClassifyInput) -> MoveCategory {
    if input.in_book && input.ply <= 24 {
        return MoveCategory::Book;
    }
    if input.legal_moves <= 1 {
        return MoveCategory::Forced;
    }
    if input.missed_mate || input.allowed_mate {
        return MoveCategory::Blunder;
    }
    if input.delta_w <= 1.5 && input.sacrifice && input.played_win >= 48.0 && input.best_win <= 88.0
    {
        return MoveCategory::Brilliant;
    }
    if input.delta_w <= 2.0 && input.second_gap >= 15.0 {
        return MoveCategory::Great;
    }
    if input.played_is_best || input.delta_w <= 0.5 {
        return MoveCategory::Best;
    }
    if input.delta_w <= 2.5 {
        return MoveCategory::Excellent;
    }
    if input.delta_w <= 6.0 {
        return MoveCategory::Good;
    }
    if input.prev_opponent_blunder
        && input.best_win >= 70.0
        && input.delta_w > 12.0
        && input.played_win >= 45.0
    {
        return MoveCategory::Miss;
    }
    if input.delta_w <= 12.0 {
        return MoveCategory::Inaccuracy;
    }
    if input.delta_w <= 22.0 {
        return MoveCategory::Mistake;
    }
    MoveCategory::Blunder
}

/// Critical position: gap between the first and second line ≥ 10% WinProb.
pub fn is_critical(second_gap: f64) -> bool {
    second_gap >= 10.0
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "classify_tests.rs"]
mod tests;
