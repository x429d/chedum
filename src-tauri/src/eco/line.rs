use shakmaty::{san::SanPlus, Chess, Position};

use crate::domain::position::{book_key, position_fen};

pub(super) fn line_positions(pgn: &str) -> Vec<(String, usize)> {
    let mut pos = Chess::default();
    let mut out = Vec::new();
    for token in pgn.split_whitespace() {
        if token.ends_with('.') || matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*") {
            continue;
        }
        let clean = token.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
        if clean.is_empty() {
            continue;
        }
        let parsed = match SanPlus::from_ascii(clean.as_bytes()) {
            Ok(parsed) => parsed,
            Err(_) => break,
        };
        let mv = match parsed.san.to_move(&pos) {
            Ok(mv) => mv,
            Err(_) => break,
        };
        pos.play_unchecked(mv);
        let ply = out.len() + 1;
        out.push((book_key(&position_fen(&pos)), ply));
    }
    out
}
