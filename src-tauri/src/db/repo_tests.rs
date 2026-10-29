use super::*;
use crate::db::analyses_repo::{
    backfill_key_moments, backfill_lichess_accuracy, backfill_summaries, get_analysis, get_moves,
    insert_analysis_draft, save_move_comment,
};
use crate::db::connection::Db;
use crate::db::stats_repo::hub_stats;
use crate::domain::analysis::{
    AnalysisDraft, CategoryCounts, GameAnalysis, MoveCategory, MoveEvaluation, Phase, PvLine,
    TimeFlag,
};
use crate::domain::game::{Color, GameDraft, GameSource, TimeClass, UserColor};
const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn draft(
    external_id: &str,
    white: &str,
    black: &str,
    user_color: UserColor,
    result: &str,
    time_class: TimeClass,
    played_at: i64,
) -> GameDraft {
    GameDraft {
        external_id: external_id.to_string(),
        source: GameSource::Manual,
        pgn: "1. e4 e5 *".to_string(),
        initial_fen: None,
        white_username: white.to_string(),
        black_username: black.to_string(),
        white_elo: Some(1800),
        black_elo: Some(1700),
        user_color,
        result: result.to_string(),
        termination: Some("Normal".to_string()),
        time_control: Some("600+0".to_string()),
        time_class: Some(time_class),
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

fn move_eval(
    game_id: i64,
    ply: i32,
    phase: Phase,
    classification: MoveCategory,
    flag: TimeFlag,
) -> MoveEvaluation {
    MoveEvaluation {
        game_id,
        ply,
        phase,
        fen_before: START.to_string(),
        fen_after: START.to_string(),
        played_uci: "e2e4".to_string(),
        played_san: "e4".to_string(),
        best_uci: Some("e2e4".to_string()),
        best_san: Some("e4".to_string()),
        eval_cp: Some(20),
        eval_mate: None,
        best_eval_cp: Some(25),
        best_eval_mate: None,
        win_prob_white: 55.0,
        win_prob_loss: 1.0,
        move_accuracy: 97.0,
        classification,
        is_critical: false,
        is_key_moment: false,
        material_diff_white: 0,
        clock_remaining_sec: Some(500.0),
        time_spent_sec: Some(5.0),
        time_flag: flag,
        explanation_tag: None,
        explanation_text: None,
        pv_lines: vec![PvLine {
            rank: 1,
            score_cp: Some(25),
            score_mate: None,
            win_prob: 55.0,
            uci_moves: vec!["e2e4".to_string()],
            san_moves: vec!["e4".to_string()],
        }],
        user_comment: String::new(),
    }
}

fn analysis(
    game_id: i64,
    white_accuracy: f64,
    black_accuracy: f64,
    moves: Vec<MoveEvaluation>,
) -> AnalysisDraft {
    let mut counts = CategoryCounts::new();
    counts.increment(Color::White, MoveCategory::Best);
    counts.increment(Color::Black, MoveCategory::Inaccuracy);
    AnalysisDraft {
        analysis: GameAnalysis {
            game_id,
            engine_name: "Stockfish 19".to_string(),
            engine_version: "19".to_string(),
            classifier_version: crate::domain::analysis::CLASSIFIER_VERSION.to_string(),
            accuracy_method_version: crate::domain::analysis::ACCURACY_METHOD_VERSION.to_string(),
            depth: 14,
            multipv: 2,
            analyzed_at: "2026-01-01T00:00:00Z".to_string(),
            white_accuracy: Some(white_accuracy),
            black_accuracy: Some(black_accuracy),
            white_acpl: Some(20.0),
            black_acpl: Some(30.0),
            white_perf_elo: Some(2000),
            black_perf_elo: Some(1800),
            white_opening_acc: Some(white_accuracy),
            white_middlegame_acc: None,
            white_endgame_acc: None,
            black_opening_acc: Some(black_accuracy),
            black_middlegame_acc: None,
            black_endgame_acc: None,
            counts,
            summary_text: "Тестовое резюме".to_string(),
            key_moments_count: 1,
        },
        moves,
    }
}

#[test]
fn inserts_dedupes_and_filters_games() {
    let db = Db::in_memory().unwrap();
    let first = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let second = draft(
        "g2",
        "bob",
        "alice",
        UserColor::Black,
        "1-0",
        TimeClass::Blitz,
        200,
    );
    let id1 = db.with_tx(|tx| insert_game(tx, &first)).unwrap().unwrap();
    let id2 = db.with_tx(|tx| insert_game(tx, &second)).unwrap().unwrap();
    assert!(db.with_tx(|tx| insert_game(tx, &first)).unwrap().is_none());
    assert_eq!(count_games(&db).unwrap(), 2);

    let all = list_games(&db, &GameFilter::default()).unwrap();
    assert_eq!(all.total, 2);
    assert_eq!(all.games[0].id, id2);

    let white = list_games(
        &db,
        &GameFilter {
            color: Some(UserColor::White),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(white.games.len(), 1);
    assert_eq!(white.games[0].id, id1);

    let blitz = list_games(
        &db,
        &GameFilter {
            time_class: Some(TimeClass::Blitz),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(blitz.games.len(), 1);

    let wins = list_games(
        &db,
        &GameFilter {
            result: Some("win".to_string()),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(wins.games.len(), 1);
    assert_eq!(wins.games[0].id, id1);

    let game = get_game(&db, id1).unwrap();
    assert_eq!(game.white_username, "alice");
    assert_eq!(game.user_result(), Some("win"));
}
#[test]
fn stores_analysis_and_move_comments() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let moves = vec![
        move_eval(id, 1, Phase::Opening, MoveCategory::Book, TimeFlag::Normal),
        move_eval(
            id,
            2,
            Phase::Opening,
            MoveCategory::Inaccuracy,
            TimeFlag::TimeTrouble,
        ),
    ];
    insert_analysis_draft(&db, &analysis(id, 90.0, 80.0, moves)).unwrap();
    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert_eq!(stored.white_accuracy, Some(90.0));
    assert_eq!(stored.counts.get(Color::White, MoveCategory::Best), 1);
    let moves = get_moves(&db, id).unwrap();
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[1].time_flag, TimeFlag::TimeTrouble);
    save_move_comment(&db, id, 1, "Хороший ход").unwrap();
    assert_eq!(get_moves(&db, id).unwrap()[0].user_comment, "Хороший ход");
}

#[test]
fn aggregates_hub_statistics() {
    let db = Db::in_memory().unwrap();
    let white_game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let black_game = draft(
        "g2",
        "bob",
        "alice",
        UserColor::Black,
        "1-0",
        TimeClass::Blitz,
        200,
    );
    let id1 = db
        .with_tx(|tx| insert_game(tx, &white_game))
        .unwrap()
        .unwrap();
    let id2 = db
        .with_tx(|tx| insert_game(tx, &black_game))
        .unwrap()
        .unwrap();
    insert_analysis_draft(
        &db,
        &analysis(
            id1,
            90.0,
            80.0,
            vec![move_eval(
                id1,
                1,
                Phase::Opening,
                MoveCategory::Best,
                TimeFlag::TimeTrouble,
            )],
        ),
    )
    .unwrap();
    insert_analysis_draft(
        &db,
        &analysis(
            id2,
            70.0,
            60.0,
            vec![move_eval(
                id2,
                2,
                Phase::Opening,
                MoveCategory::Inaccuracy,
                TimeFlag::ImpulsiveError,
            )],
        ),
    )
    .unwrap();
    let stats = hub_stats(&db).unwrap();
    assert_eq!(stats.kpis.total_games, 2);
    assert_eq!(stats.kpis.wins, 1);
    assert_eq!(stats.kpis.losses, 1);
    assert!((stats.kpis.avg_accuracy.unwrap() - 75.0).abs() < 1e-9);
    assert!(!stats.openings.is_empty());
    let opening = &stats.openings[0];
    assert_eq!(opening.games, 2);
    assert_eq!(opening.analyzed, 2);
    assert!(!opening.reliable);
    assert!(opening.price > 0.0);
    assert_eq!(stats.phases[0].moves, 2);
    assert_eq!(stats.phases[0].games, 2);
    assert_eq!(stats.trend.len(), 2);
    assert_eq!(stats.time.time_trouble_moves, 1);
    assert_eq!(stats.time.impulsive_errors, 1);
    // Меньше 20 разобранных партий — оценка силы по точности не показывается.
    assert!(stats.kpis.avg_perf_elo.is_none());
    assert!(stats.kpis.avg_perf_elo_spread.is_none());
    let config = stats.config.as_ref().unwrap();
    assert_eq!(config.depth, 14);
    assert_eq!(config.included, 2);
    assert_eq!(config.total, 2);
    assert!(config.mismatched_game_ids.is_empty());
}

#[test]
fn hub_ignores_analyses_from_another_configuration() {
    let db = Db::in_memory().unwrap();
    let mut ids = Vec::new();
    for index in 0..2 {
        let game = draft(
            &format!("g{index}"),
            "alice",
            "bob",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            index,
        );
        let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
        insert_analysis_draft(
            &db,
            &analysis(
                id,
                90.0,
                80.0,
                vec![move_eval(
                    id,
                    1,
                    Phase::Opening,
                    MoveCategory::Best,
                    TimeFlag::Normal,
                )],
            ),
        )
        .unwrap();
        ids.push(id);
    }
    let before = hub_stats(&db).unwrap();
    let before_accuracy = before.kpis.avg_accuracy.unwrap();
    assert_eq!(before.kpis.analyzed_games, 2);

    let deep_game = draft(
        "g-deep",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        5,
    );
    let deep_id = db
        .with_tx(|tx| insert_game(tx, &deep_game))
        .unwrap()
        .unwrap();
    let mut deep = analysis(
        deep_id,
        10.0,
        10.0,
        vec![move_eval(
            deep_id,
            1,
            Phase::Opening,
            MoveCategory::Blunder,
            TimeFlag::Normal,
        )],
    );
    deep.analysis.depth = 18;
    insert_analysis_draft(&db, &deep).unwrap();

    let after = hub_stats(&db).unwrap();
    assert!((after.kpis.avg_accuracy.unwrap() - before_accuracy).abs() < 1e-9);
    assert_eq!(after.kpis.analyzed_games, 2);
    let config = after.config.as_ref().unwrap();
    assert_eq!(config.depth, 14);
    assert_eq!(config.included, 2);
    assert_eq!(config.total, 3);
    assert_eq!(config.mismatched_game_ids, vec![deep_id]);
    assert_eq!(ids.len(), 2);
}

#[test]
fn performance_estimate_requires_twenty_analyses() {
    let db = Db::in_memory().unwrap();
    for index in 0..20 {
        let game = draft(
            &format!("g{index}"),
            "alice",
            "bob",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            index,
        );
        let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
        insert_analysis_draft(
            &db,
            &analysis(
                id,
                70.0 + index as f64,
                60.0,
                vec![move_eval(
                    id,
                    1,
                    Phase::Opening,
                    MoveCategory::Best,
                    TimeFlag::Normal,
                )],
            ),
        )
        .unwrap();
    }
    let stats = hub_stats(&db).unwrap();
    assert_eq!(stats.kpis.analyzed_games, 20);
    assert!(stats.kpis.avg_perf_elo.is_some());
    assert!(stats.kpis.avg_perf_elo_spread.is_some());
    let opening = &stats.openings[0];
    assert_eq!(opening.analyzed, 20);
    assert!(opening.reliable);
    assert!(opening.price > 0.0);
}

#[test]
fn backfills_accuracy_with_lichess_method() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let mut first = move_eval(id, 1, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    first.best_eval_cp = Some(50);
    first.win_prob_white = 55.0;
    let mut second = move_eval(id, 2, Phase::Opening, MoveCategory::Blunder, TimeFlag::Normal);
    second.best_eval_cp = Some(-200);
    second.win_prob_white = 40.0;
    insert_analysis_draft(&db, &analysis(id, 90.0, 80.0, vec![first, second])).unwrap();

    assert_eq!(backfill_lichess_accuracy(&db, false).unwrap(), 1);
    let stored = get_analysis(&db, id).unwrap().unwrap();
    // Белые не потеряли Win% ни на одном ходу — по методу Lichess это ровно 100.
    assert_eq!(stored.white_accuracy, Some(100.0));
    assert_eq!(stored.white_opening_acc, Some(100.0));
    let black = stored.black_accuracy.unwrap();
    assert!(black > 0.0 && black < 100.0);
    assert_ne!(stored.black_accuracy, Some(80.0));
    // Повторный запуск идемпотентен.
    assert_eq!(backfill_lichess_accuracy(&db, false).unwrap(), 0);
}

#[test]
fn backfill_selects_key_moments_and_updates_summary() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let mut moves = Vec::new();
    for ply in 1..=30 {
        let white_moment = ply % 4 == 1;
        let mut mv = move_eval(
            id,
            ply,
            Phase::Middlegame,
            if white_moment {
                MoveCategory::Blunder
            } else {
                MoveCategory::Best
            },
            TimeFlag::Normal,
        );
        mv.win_prob_loss = if white_moment { 25.0 } else { 0.0 };
        mv.pv_lines[0].win_prob = if white_moment { 70.0 } else { 50.0 };
        moves.push(mv);
    }
    insert_analysis_draft(&db, &analysis(id, 70.0, 80.0, moves)).unwrap();
    assert_eq!(backfill_key_moments(&db).unwrap(), 1);
    let selected: Vec<_> = get_moves(&db, id)
        .unwrap()
        .into_iter()
        .filter(|mv| mv.is_key_moment)
        .collect();
    assert_eq!(selected.len(), 8);
    for pair in selected.windows(2) {
        assert!(pair[1].ply - pair[0].ply >= 3);
    }
    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert_eq!(stored.key_moments_count, 8);
    assert!(stored.summary_text.contains("Ключевых моментов: 8"));
    assert_eq!(backfill_key_moments(&db).unwrap(), 0);
}

#[test]
fn backfill_keeps_zero_key_moments_for_a_clean_game() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let moves = (1..=20)
        .map(|ply| {
            move_eval(
                id,
                ply,
                Phase::Middlegame,
                MoveCategory::Best,
                TimeFlag::Normal,
            )
        })
        .collect();
    let mut draft = analysis(id, 95.0, 90.0, moves);
    draft.analysis.key_moments_count = 7;
    insert_analysis_draft(&db, &draft).unwrap();
    assert_eq!(backfill_key_moments(&db).unwrap(), 1);
    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert_eq!(stored.key_moments_count, 0);
    assert!(!get_moves(&db, id).unwrap().iter().any(|mv| mv.is_key_moment));
}

#[test]
fn summary_text_always_matches_accuracy_fields() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    // Старая запись хранит резюме, посчитанное до смены алгоритма.
    let mut stale = analysis(
        id,
        63.4,
        59.2,
        vec![move_eval(
            id,
            1,
            Phase::Opening,
            MoveCategory::Blunder,
            TimeFlag::TimeTrouble,
        )],
    );
    stale.analysis.summary_text =
        "Точность ваших ходов — 74.6%, ACPL 101.6.".to_string();
    insert_analysis_draft(&db, &stale).unwrap();

    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert!(stored.summary_text.contains("63.4%"), "{}", stored.summary_text);
    assert!(!stored.summary_text.contains("74.6"), "{}", stored.summary_text);
    assert!(stored.summary_text.contains("ACPL 20.0"), "{}", stored.summary_text);

    assert_eq!(backfill_summaries(&db).unwrap(), 1);
    let raw: String = db
        .with(|conn| {
            Ok(conn.query_row(
                "SELECT summary_text FROM game_analyses WHERE game_id = ?1",
                [id],
                |row| row.get(0),
            )?)
        })
        .unwrap();
    assert!(raw.contains("63.4%"), "{raw}");
    assert!(!raw.contains("74.6"), "{raw}");
    assert_eq!(backfill_summaries(&db).unwrap(), 0);
}
