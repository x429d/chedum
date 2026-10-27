use crate::db::connection::Db;
use crate::db::games_repo::{insert_game, list_games};
use crate::domain::game::{GameDraft, GameFilter, GameSource, TimeClass, UserColor};

fn draft(external_id: &str, played_at: i64) -> GameDraft {
    GameDraft {
        external_id: external_id.to_string(),
        source: GameSource::Manual,
        pgn: "1. e4 e5 *".to_string(),
        initial_fen: None,
        white_username: "alice".to_string(),
        black_username: "bob".to_string(),
        white_elo: Some(1800),
        black_elo: Some(1700),
        user_color: UserColor::White,
        result: "1-0".to_string(),
        termination: Some("Normal".to_string()),
        time_control: Some("600+0".to_string()),
        time_class: Some(TimeClass::Rapid),
        eco: Some("C41".to_string()),
        opening_name: Some("Philidor Defense".to_string()),
        played_at,
        total_plies: 2,
        notes: String::new(),
        is_favorite: false,
    }
}

#[test]
fn paginates_game_history_with_limit_and_offset() {
    let db = Db::in_memory().unwrap();
    for (index, played_at) in [100, 200, 300].iter().enumerate() {
        let game = draft(&format!("g{index}"), *played_at);
        db.with_tx(|tx| insert_game(tx, &game)).unwrap();
    }
    let first = list_games(
        &db,
        &GameFilter {
            limit: Some(2),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(first.total, 3);
    assert_eq!(first.games.len(), 2);
    let second = list_games(
        &db,
        &GameFilter {
            limit: Some(2),
            offset: Some(2),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(second.total, 3);
    assert_eq!(second.games.len(), 1);
    assert_eq!(second.offset, 2);
}
