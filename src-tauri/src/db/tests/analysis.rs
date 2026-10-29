use super::support::{analysis, draft, move_eval};
use crate::db::analyses_repo::{
    backfill_key_moments, backfill_lichess_accuracy, backfill_summaries, get_analysis, get_moves,
    insert_analysis_draft, save_move_comment,
};
use crate::db::connection::Db;
use crate::db::games_repo::{get_game, insert_game, list_games};
use crate::domain::analysis::{MoveCategory, Phase, TimeFlag};
use crate::domain::game::{Color, GameFilter, TimeClass, UserColor};

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
fn export_pgn_keeps_game_importable_and_adds_annotator() {
    let db = Db::in_memory().unwrap();
    let mut game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    game.pgn = "[White \"alice\"]\n[Black \"bob\"]\n[Result \"1-0\"]\n\n1. e4 e5 2. Nf3 Nc6 1-0"
        .to_string();
    game.total_plies = 4;
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

    let exported = crate::service::import::export_pgn(&db, &[id]).unwrap();
    assert!(
        exported.contains("[Annotator \"Chenal · Stockfish 19 · depth 14 · Lichess accuracy\"]"),
        "{exported}"
    );

    let fresh = Db::in_memory().unwrap();
    let summary = crate::service::import::import_pgn_text(&fresh, &exported, "alice").unwrap();
    assert_eq!(summary.imported, 1, "{:?}", summary.errors);
    let page = list_games(&fresh, &GameFilter::default()).unwrap();
    let imported = get_game(&fresh, page.games[0].id).unwrap();
    let original = crate::pgn::parse_single(&exported).unwrap();
    let reimported = crate::pgn::parse_single(&imported.pgn).unwrap();
    assert_eq!(original.moves.len(), reimported.moves.len());
    assert_eq!(original.result, reimported.result);
    assert_eq!(imported.result, "1-0");
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
