use std::collections::HashMap;

use crate::domain::game::Game;

/// Экспортирует партии в один PGN-документ, разделяя их пустой строкой.
pub fn export_games(games: &[Game]) -> String {
    export_games_with_annotations(games, &HashMap::new())
}

/// То же, но с подписью происхождения анализа в заголовке `Annotator`.
pub fn export_games_with_annotations(
    games: &[Game],
    annotations: &HashMap<i64, String>,
) -> String {
    let mut out = String::new();
    for game in games {
        let pgn = game.pgn.trim_end();
        let annotated = match annotations.get(&game.id) {
            Some(value) => apply_annotator(pgn, value),
            None => pgn.to_string(),
        };
        out.push_str(annotated.trim_end());
        out.push('\n');
        out.push('\n');
    }
    out
}

/// Вставляет или заменяет `[Annotator "…"]`, не ломая остальные заголовки.
fn apply_annotator(pgn: &str, value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let line = format!("[Annotator \"{escaped}\"]");
    let mut lines: Vec<String> = pgn.lines().map(str::to_string).collect();
    lines.retain(|existing| !existing.starts_with("[Annotator "));
    let insert_at = lines
        .iter()
        .position(|existing| !existing.starts_with('['))
        .unwrap_or(lines.len());
    lines.insert(insert_at, line);
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::game::{GameSource, TimeClass, UserColor};

    fn game(pgn: &str) -> Game {
        Game {
            id: 1,
            external_id: "x".to_string(),
            source: GameSource::Manual,
            pgn: pgn.to_string(),
            initial_fen: None,
            white_username: "A".to_string(),
            black_username: "B".to_string(),
            white_elo: None,
            black_elo: None,
            user_color: UserColor::White,
            result: "1-0".to_string(),
            termination: None,
            time_control: None,
            time_class: Some(TimeClass::Rapid),
            eco: None,
            opening_name: None,
            opening_ply: None,
            opening_source: None,
            played_at: 0,
            total_plies: 1,
            notes: String::new(),
            is_favorite: false,
        }
    }

    #[test]
    fn exports_multiple_games() {
        let out = export_games(&[game("1. e4 *"), game("1. d4 *")]);
        assert!(out.contains("1. e4 *"));
        assert!(out.contains("1. d4 *"));
        assert!(out.ends_with("\n\n"));
    }

    #[test]
    fn adds_annotator_header_without_touching_movetext() {
        let mut annotations = HashMap::new();
        annotations.insert(1, "Chenal · Stockfish 19 · depth 14 · Lichess accuracy".to_string());
        let out = export_games_with_annotations(
            &[game("[White \"A\"]\n[Black \"B\"]\n\n1. e4 e5 *")],
            &annotations,
        );
        assert!(out.contains("[Annotator \"Chenal · Stockfish 19 · depth 14 · Lichess accuracy\"]"));
        assert!(out.contains("1. e4 e5 *"));
    }

    #[test]
    fn replaces_existing_annotator_instead_of_duplicating() {
        let mut annotations = HashMap::new();
        annotations.insert(1, "Chenal · Stockfish 19 · depth 14 · Lichess accuracy".to_string());
        let out = export_games_with_annotations(
            &[game("[White \"A\"]\n[Annotator \"old\"]\n\n1. e4 *")],
            &annotations,
        );
        assert_eq!(out.matches("[Annotator ").count(), 1);
        assert!(!out.contains("\"old\""));
    }
}
