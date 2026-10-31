use shakmaty::Position;

use crate::domain::position::parse_fen;
use crate::pgn::parser::ParsedGame;

/// Serializes a parsed game back to PGN, preserving tags and comments.
pub fn game_to_pgn(game: &ParsedGame) -> String {
    let mut headers: Vec<(String, String)> = Vec::new();
    const ORDER: [&str; 18] = [
        "Event",
        "Site",
        "Date",
        "Round",
        "White",
        "Black",
        "Result",
        "WhiteElo",
        "BlackElo",
        "TimeControl",
        "Termination",
        "ECO",
        "Opening",
        "UTCDate",
        "UTCTime",
        "FEN",
        "SetUp",
        "Variant",
    ];
    for key in ORDER {
        if let Some(value) = game.tags.get(key) {
            headers.push((key.to_string(), value.clone()));
        }
    }
    for (key, value) in &game.tags {
        if !ORDER.contains(&key.as_str()) {
            headers.push((key.clone(), value.clone()));
        }
    }
    if !headers.iter().any(|(key, _)| key == "Result") {
        headers.push(("Result".to_string(), game.result.clone()));
    }
    if game.initial_fen.is_some() && !headers.iter().any(|(key, _)| key == "SetUp") {
        headers.push(("SetUp".to_string(), "1".to_string()));
    }
    let mut out = String::new();
    for (key, value) in headers {
        out.push_str(&format!("[{key} \"{}\"]\n", escape_tag(&value)));
    }
    out.push('\n');
    let black_to_move = game
        .initial_fen
        .as_deref()
        .and_then(|fen| parse_fen(fen).ok())
        .map(|pos| pos.turn() == shakmaty::Color::Black)
        .unwrap_or(false);
    for (index, mv) in game.moves.iter().enumerate() {
        let white_move = if black_to_move {
            index % 2 == 1
        } else {
            index % 2 == 0
        };
        let number = if black_to_move {
            (index + 2) / 2
        } else {
            index / 2 + 1
        };
        if white_move {
            out.push_str(&format!("{number}. "));
        } else if index == 0 {
            out.push_str(&format!("{number}... "));
        }
        out.push_str(&mv.san);
        out.push(' ');
        for comment in &mv.comments {
            out.push_str(&format!("{{{comment}}} "));
        }
    }
    out.push_str(&game.result);
    out.push('\n');
    out
}

fn escape_tag(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
