use crate::domain::analysis::Phase;
use crate::domain::position::{
    developed_count, heavy_material_total, parse_fen, piece_count_nonpawn,
};

/// Determines the game phase from the ply number and position.
///
/// - endgame: total value of non-pawn, non-king pieces ≤ 26;
/// - opening: ply ≤ 24 or fewer than 6 pieces developed, provided more than 10 pieces remain;
/// - otherwise middlegame.
pub fn phase_for(ply: u32, fen: &str) -> Phase {
    let pos = match parse_fen(fen) {
        Ok(pos) => pos,
        Err(_) => {
            return if ply <= 24 {
                Phase::Opening
            } else {
                Phase::Middlegame
            }
        }
    };
    if heavy_material_total(&pos) <= 26 {
        return Phase::Endgame;
    }
    let pieces = piece_count_nonpawn(&pos);
    if pieces > 10 && (ply <= 24 || developed_count(&pos) < 6) {
        Phase::Opening
    } else {
        Phase::Middlegame
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    const ENDGAME: &str = "8/5pk1/8/8/8/6K1/5P2/8 w - - 0 40";
    const MIDDLE: &str = "r1bq1rk1/pp2ppbp/2np1np1/8/3NP3/2N1BP2/PPPQ2PP/R3KB1R w KQ - 0 10";

    #[test]
    fn start_is_opening() {
        assert_eq!(phase_for(1, START), Phase::Opening);
    }

    #[test]
    fn low_material_is_endgame() {
        assert_eq!(phase_for(80, ENDGAME), Phase::Endgame);
    }

    #[test]
    fn developed_position_after_24_is_middlegame() {
        assert_eq!(phase_for(30, MIDDLE), Phase::Middlegame);
    }

    #[test]
    fn invalid_fen_falls_back_to_ply() {
        assert_eq!(phase_for(4, "not a fen"), Phase::Opening);
        assert_eq!(phase_for(40, "not a fen"), Phase::Middlegame);
    }
}
