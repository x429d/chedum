use super::support::{analysis, draft, move_eval};
use crate::db::accounts_repo;
use crate::db::analyses_repo::insert_analysis_draft;
use crate::db::connection::Db;
use crate::db::games_repo::insert_game;
use crate::db::settings_repo::set_setting;
use crate::db::stats_repo::hub_stats;
use crate::domain::analysis::{MoveCategory, Phase, TimeFlag};
use crate::domain::game::{GameSource, TimeClass, UserColor};
use crate::domain::AccountPlatform;

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
    let stats = hub_stats(&db, None).unwrap();
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
    let before = hub_stats(&db, None).unwrap();
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

    let after = hub_stats(&db, None).unwrap();
    assert!((after.kpis.avg_accuracy.unwrap() - before_accuracy).abs() < 1e-9);
    assert_eq!(after.kpis.analyzed_games, 2);
    let config = after.config.as_ref().unwrap();
    assert_eq!(config.depth, 14);
    assert_eq!(config.included, 2);
    assert_eq!(config.total, 3);
    assert_eq!(config.mismatched_game_ids, vec![deep_id]);
    assert!(
        config.upgrade_game_ids.is_empty(),
        "более глубокий анализ не должен предлагаться к пересчёту"
    );
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
    let stats = hub_stats(&db, None).unwrap();
    assert_eq!(stats.kpis.analyzed_games, 20);
    assert!(stats.kpis.avg_perf_elo.is_some());
    assert!(stats.kpis.avg_perf_elo_spread.is_some());
    let opening = &stats.openings[0];
    assert_eq!(opening.analyzed, 20);
    assert!(opening.reliable);
    assert!(opening.price > 0.0);
}


#[test]
fn performance_elo_is_split_by_account_and_platform() {
    let db = Db::in_memory().unwrap();
    let chess = accounts_repo::create(&db, AccountPlatform::Chesscom, "alice").unwrap();
    let lichess = accounts_repo::create(&db, AccountPlatform::Lichess, "alice_li").unwrap();
    let insert = |prefix: &str,
                  id: i64,
                  account: i64,
                  source: GameSource,
                  user_elo: i32,
                  opponent_elo: i32,
                  perf_elo: i32| {
        let mut game = draft(
            &format!("{prefix}-{id}"),
            "alice",
            "bob",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            id,
        );
        game.source = source;
        game.account_id = Some(account);
        game.white_elo = Some(user_elo);
        game.black_elo = Some(opponent_elo);
        let game_id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
        let mut analysis = analysis(
            game_id,
            80.0,
            70.0,
            vec![move_eval(
                game_id,
                1,
                Phase::Opening,
                MoveCategory::Best,
                TimeFlag::Normal,
            )],
        );
        analysis.analysis.white_perf_elo = Some(perf_elo);
        insert_analysis_draft(&db, &analysis).unwrap();
    };
    for index in 0..20 {
        insert(
            "chess",
            index,
            chess.id,
            GameSource::Chesscom,
            1500,
            1600,
            1550,
        );
        insert(
            "lichess",
            index,
            lichess.id,
            GameSource::Lichess,
            2000,
            2100,
            2050,
        );
    }

    let stats = hub_stats(&db, None).unwrap();
    assert_eq!(stats.ratings.len(), 2);
    let chess_rating = stats
        .ratings
        .iter()
        .find(|rating| rating.platform == GameSource::Chesscom)
        .unwrap();
    assert_eq!(chess_rating.account_username.as_deref(), Some("alice"));
    assert_eq!(chess_rating.games, 20);
    assert_eq!(chess_rating.analyzed, 20);
    assert_eq!(chess_rating.avg_user_elo, Some(1500.0));
    assert_eq!(chess_rating.avg_opponent_elo, Some(1600.0));
    assert_eq!(chess_rating.perf_elo, Some(1550.0));
    let lichess_rating = stats
        .ratings
        .iter()
        .find(|rating| rating.platform == GameSource::Lichess)
        .unwrap();
    assert_eq!(lichess_rating.avg_user_elo, Some(2000.0));
    assert_eq!(lichess_rating.avg_opponent_elo, Some(2100.0));
    assert_eq!(lichess_rating.perf_elo, Some(2050.0));
    assert!(
        stats.kpis.avg_perf_elo.is_none(),
        "mixed-platform performance must not be shown"
    );

    let chess_stats = hub_stats(&db, Some(chess.id)).unwrap();
    assert_eq!(chess_stats.ratings.len(), 1);
    assert_eq!(chess_stats.ratings[0].perf_elo, Some(1550.0));
    assert_eq!(chess_stats.kpis.avg_perf_elo, Some(1550.0));
}

#[test]
fn reanalysis_offered_only_for_shallower_analyses() {
    let db = Db::in_memory().unwrap();
    let mut shallow_ids = Vec::new();
    for index in 0..2 {
        let game = draft(
            &format!("shallow-{index}"),
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
        shallow_ids.push(id);
    }
    let deep_game = draft(
        "deep",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        10,
    );
    let deep_id = db
        .with_tx(|tx| insert_game(tx, &deep_game))
        .unwrap()
        .unwrap();
    let mut deep = analysis(
        deep_id,
        90.0,
        80.0,
        vec![move_eval(
            deep_id,
            1,
            Phase::Opening,
            MoveCategory::Best,
            TimeFlag::Normal,
        )],
    );
    deep.analysis.depth = 18;
    insert_analysis_draft(&db, &deep).unwrap();

    // Текущие настройки 14: глубокий анализ 18 не предлагается к пересчёту.
    let shallow_target = hub_stats(&db, None).unwrap();
    assert!(shallow_target
        .config
        .as_ref()
        .unwrap()
        .upgrade_game_ids
        .is_empty());

    set_setting(&db, "analysis_depth", "18").unwrap();
    let deep_target = hub_stats(&db, None).unwrap();
    assert_eq!(
        deep_target.config.as_ref().unwrap().upgrade_game_ids,
        shallow_ids,
        "при глубине 18 пересчитываются только партии, разобранные на 14"
    );
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
    let undated_id = db.with_tx(|tx| insert_game(tx, &undated)).unwrap().unwrap();
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

    let stats = hub_stats(&db, None).unwrap();
    assert_eq!(stats.trend.len(), 200);
    assert_eq!(stats.trend.first().unwrap().played_at, 101);
    assert_eq!(stats.trend.last().unwrap().played_at, 300);
    assert!(stats
        .trend
        .windows(2)
        .all(|pair| pair[0].played_at < pair[1].played_at));
    assert!(!stats.trend.iter().any(|point| point.played_at == 0));
}
