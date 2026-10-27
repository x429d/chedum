use crate::db::analyses_repo::{get_analysis, get_moves, insert_analysis_draft};
use crate::db::connection::Db;
use crate::db::games_delete::delete_games;
use crate::db::games_repo::{count_games, insert_game};
use crate::domain::analysis::{AnalysisDraft, CategoryCounts, GameAnalysis};
use crate::domain::game::{GameDraft, GameSource, TimeClass, UserColor};

fn draft(external_id: &str, white: &str, black: &str, played_at: i64) -> GameDraft {
    GameDraft {
        external_id: external_id.to_string(),
        source: GameSource::Manual,
        pgn: "1. e4 e5 *".to_string(),
        initial_fen: None,
        white_username: white.to_string(),
        black_username: black.to_string(),
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

fn analysis(game_id: i64) -> AnalysisDraft {
    AnalysisDraft {
        analysis: GameAnalysis {
            game_id,
            engine_name: "Stockfish 19".to_string(),
            depth: 14,
            multipv: 2,
            analyzed_at: "2026-01-01T00:00:00Z".to_string(),
            white_accuracy: Some(90.0),
            black_accuracy: Some(80.0),
            white_acpl: Some(20.0),
            black_acpl: Some(30.0),
            white_perf_elo: Some(2000),
            black_perf_elo: Some(1800),
            white_opening_acc: Some(90.0),
            white_middlegame_acc: None,
            white_endgame_acc: None,
            black_opening_acc: Some(80.0),
            black_middlegame_acc: None,
            black_endgame_acc: None,
            counts: CategoryCounts::new(),
            summary_text: "Тест".to_string(),
            key_moments_count: 0,
        },
        moves: Vec::new(),
    }
}

#[test]
fn deletes_games_in_batch_and_cascades_analysis() {
    let db = Db::in_memory().unwrap();
    let first = draft("g1", "alice", "bob", 100);
    let second = draft("g2", "bob", "alice", 200);
    let id1 = db.with_tx(|tx| insert_game(tx, &first)).unwrap().unwrap();
    let id2 = db.with_tx(|tx| insert_game(tx, &second)).unwrap().unwrap();
    insert_analysis_draft(&db, &analysis(id1)).unwrap();
    assert!(get_analysis(&db, id1).unwrap().is_some());
    assert_eq!(delete_games(&db, &[id1, id2]).unwrap(), 2);
    assert_eq!(count_games(&db).unwrap(), 0);
    assert!(get_analysis(&db, id1).unwrap().is_none());
    assert!(get_moves(&db, id1).unwrap().is_empty());
}
