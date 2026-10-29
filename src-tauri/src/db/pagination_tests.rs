use crate::db::connection::Db;
use crate::db::games_repo::{insert_game, list_games, pending_analysis, set_favorite};
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
        opening_ply: Some(3),
        opening_source: Some("position".to_string()),
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

fn analyze(db: &Db, game_id: i64) {
    db.with_tx(|tx| {
        tx.execute(
            "INSERT INTO game_analyses (game_id, engine_name, engine_version, depth, multipv, \
             classifier_version, accuracy_method_version, config_key, analyzed_at, \
             counts_json, summary_text, key_moments_count) \
             VALUES (?1, 'Stockfish 19', '19', 14, 2, 'classifier-v1', 'lichess-v1', \
             'Stockfish 19|19|14|2|classifier-v1|lichess-v1', '2026-01-01', '{}', 'x', 0)",
            [game_id],
        )?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn counts_analyzed_and_favorites_for_whole_selection() {
    let db = Db::in_memory().unwrap();
    let mut ids = Vec::new();
    for (index, played_at) in [100, 200, 300].iter().enumerate() {
        let game = draft(&format!("g{index}"), *played_at);
        ids.push(db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap());
    }
    set_favorite(&db, ids[0], true).unwrap();
    analyze(&db, ids[1]);

    let page = list_games(
        &db,
        &GameFilter {
            limit: Some(1),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.games.len(), 1);
    // Счётчики описывают всю выборку, а не одну страницу.
    assert_eq!(page.analyzed, 1);
    assert_eq!(page.favorites, 1);

    let key = "Stockfish 19|19|14|2|classifier-v1|lichess-v1";
    let pending = pending_analysis(&db, &GameFilter::default(), key).unwrap();
    assert_eq!(pending.ids.len(), 2);
    assert!(!pending.ids.contains(&ids[1]));
    // По две партии по три позиции: стартовая + два полухода.
    assert_eq!(pending.positions, 6);
}
