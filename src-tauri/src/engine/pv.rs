use crate::domain::position::uci_line_to_san;

pub fn line_to_san(fen: &str, uci_moves: &[String]) -> Vec<String> {
    uci_line_to_san(fen, uci_moves)
}

pub fn move_to_san(fen: &str, uci: &str) -> Option<String> {
    let san = uci_line_to_san(fen, &[uci.to_string()]);
    san.into_iter().next()
}
