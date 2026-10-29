use super::support::{analysis, draft, move_eval};
use crate::db::connection::Db;
use crate::db::analyses_repo::insert_analysis_draft;
use crate::db::games_repo::{
    get_game, insert_game, pending_analysis, record_analysis_failure, reset_analysis_attempts,
};
use crate::domain::analysis::{MoveCategory, Phase, TimeFlag};
use crate::domain::game::{GameFilter, TimeClass, UserColor};

#[test]
fn pending_queue_is_derived_from_the_target_configuration() {
    let db = Db::in_memory().unwrap();
    let shallow = draft(
        "g-shallow",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        10,
    );
    let deep = draft(
        "g-deep",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        20,
    );
    let shallow_id = db
        .with_tx(|tx| insert_game(tx, &shallow))
        .unwrap()
        .unwrap();
    let deep_id = db.with_tx(|tx| insert_game(tx, &deep)).unwrap().unwrap();
    let shallow_analysis = analysis(
        shallow_id,
        90.0,
        80.0,
        vec![move_eval(
            shallow_id,
            1,
            Phase::Opening,
            MoveCategory::Best,
            TimeFlag::Normal,
        )],
    );
    let shallow_key = crate::db::analyses_repo::analysis_config_key(&shallow_analysis.analysis);
    insert_analysis_draft(&db, &shallow_analysis).unwrap();
    let mut deep_analysis = analysis(
        deep_id,
        70.0,
        60.0,
        vec![move_eval(
            deep_id,
            1,
            Phase::Opening,
            MoveCategory::Best,
            TimeFlag::Normal,
        )],
    );
    deep_analysis.analysis.depth = 18;
    let deep_key = crate::db::analyses_repo::analysis_config_key(&deep_analysis.analysis);
    insert_analysis_draft(&db, &deep_analysis).unwrap();

    let shallow_queue = pending_analysis(&db, &GameFilter::default(), &shallow_key).unwrap();
    assert_eq!(shallow_queue.ids, vec![deep_id]);
    let deep_queue = pending_analysis(&db, &GameFilter::default(), &deep_key).unwrap();
    assert_eq!(deep_queue.ids, vec![shallow_id]);
}

#[test]
fn pending_queue_puts_new_games_first() {
    let db = Db::in_memory().unwrap();
    let mut ids = Vec::new();
    for (suffix, played_at) in [("early", 10), ("late", 30), ("mid", 20)] {
        let game = draft(
            suffix,
            "alice",
            "bob",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            played_at,
        );
        ids.push(db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap());
    }
    let key = "Stockfish 19|19|14|2|classifier-v1|lichess-v1";
    let queue = pending_analysis(&db, &GameFilter::default(), key).unwrap();
    assert_eq!(queue.ids, vec![ids[1], ids[2], ids[0]]);
}

#[test]
fn failed_analyses_leave_the_queue_after_three_attempts() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        10,
    );
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let key = "Stockfish 19|19|14|2|classifier-v1|lichess-v1";
    assert_eq!(
        pending_analysis(&db, &GameFilter::default(), key).unwrap().ids,
        vec![id]
    );
    for attempt in 1..=3 {
        record_analysis_failure(&db, id, &format!("сбой {attempt}")).unwrap();
    }
    let queue = pending_analysis(&db, &GameFilter::default(), key).unwrap();
    assert!(queue.ids.is_empty());
    assert_eq!(queue.excluded, 1);
    let stored = get_game(&db, id).unwrap();
    assert_eq!(stored.analysis_attempts, 3);
    reset_analysis_attempts(&db, id).unwrap();
    let queue = pending_analysis(&db, &GameFilter::default(), key).unwrap();
    assert_eq!(queue.ids, vec![id]);
    assert_eq!(queue.excluded, 0);
}

