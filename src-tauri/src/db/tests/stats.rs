use super::support::{analysis, draft, move_eval};
use crate::db::analyses_repo::insert_analysis_draft;
use crate::db::connection::Db;
use crate::db::games_repo::insert_game;
use crate::db::stats_repo::hub_stats;
use crate::domain::analysis::{MoveCategory, Phase, TimeFlag};
use crate::domain::game::{TimeClass, UserColor};

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
fn trend_keeps_last_two_hundred_games_in_chronological_order() {
    let db = Db::in_memory().unwrap();
    for index in 1..=300i64 {
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
                50.0 + (index % 50) as f64,
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
    // Партия без даты не должна попадать в тренд.
    let undated = draft(
        "g-undated",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        0,
    );
    let undated_id = db
        .with_tx(|tx| insert_game(tx, &undated))
        .unwrap()
        .unwrap();
    insert_analysis_draft(
        &db,
        &analysis(
            undated_id,
            99.0,
            60.0,
            vec![move_eval(
                undated_id,
                1,
                Phase::Opening,
                MoveCategory::Best,
                TimeFlag::Normal,
            )],
        ),
    )
    .unwrap();

    let stats = hub_stats(&db).unwrap();
    assert_eq!(stats.trend.len(), 200);
    assert_eq!(stats.trend.first().unwrap().played_at, 101);
    assert_eq!(stats.trend.last().unwrap().played_at, 300);
    assert!(stats
        .trend
        .windows(2)
        .all(|pair| pair[0].played_at < pair[1].played_at));
    assert!(!stats.trend.iter().any(|point| point.played_at == 0));
}

