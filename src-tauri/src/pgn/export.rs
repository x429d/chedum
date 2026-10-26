use crate::domain::game::Game;

/// Экспортирует партии в один PGN-документ, разделяя их пустой строкой.
pub fn export_games(games: &[Game]) -> String {
    let mut out = String::new();
    for game in games {
        let pgn = game.pgn.trim_end();
        out.push_str(pgn);
        if !pgn.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
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
}
