use super::support::{analysis, draft, move_eval};
use crate::db::analyses_repo::{
    backfill_fen_accuracy, backfill_summaries, get_analysis, insert_analysis_draft,
};
use crate::db::connection::Db;
use crate::db::games_repo::insert_game;
use crate::db::migrations;
use crate::db::settings_repo::set_setting;
use crate::db::stats_repo::hub_stats;
use crate::domain::analysis::{MoveCategory, Phase, TimeFlag};
use crate::domain::game::{Color, TimeClass, UserColor};

#[test]
fn hub_aggregates_only_the_configured_account() {
    let db = Db::in_memory().unwrap();
    set_setting(&db, "player_username", "alice").unwrap();
    let games = [
        draft(
            "a-white",
            "ALICE",
            "bob",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            100,
        ),
        draft(
            "a-black",
            "bob",
            "alice",
            UserColor::Black,
            "0-1",
            TimeClass::Blitz,
            200,
        ),
        // Игра, импортированная под другим профилем: в агрегаты alice не входит.
        draft(
            "c-white",
            "carol",
            "dave",
            UserColor::White,
            "1-0",
            TimeClass::Rapid,
            300,
        ),
    ];
    for game in &games {
        let id = db.with_tx(|tx| insert_game(tx, game)).unwrap().unwrap();
        insert_analysis_draft(
            &db,
            &analysis(
                id,
                80.0,
                70.0,
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
    assert_eq!(stats.kpis.total_games, 2);
    assert_eq!(stats.kpis.white.games, 1);
    assert_eq!(stats.kpis.black.games, 1);
    assert_eq!(stats.openings[0].games, 2);
    assert_eq!(stats.phases[0].games, 2);
    assert_eq!(stats.trend.len(), 2);
    let config = stats.config.as_ref().unwrap();
    assert_eq!(config.included, 2);
    // Анализ carol существует, но в выборку аккаунта не входит.
    assert_eq!(config.total, 2);
    assert!(config.mismatched_game_ids.is_empty());
}

#[test]
fn fen_game_uses_player_color_not_ply_parity() {
    let db = Db::in_memory().unwrap();
    let mut game = draft(
        "fen",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    // Начальная позиция с ходом чёрных: чётность полухода инвертирована.
    game.initial_fen = Some("8/8/8/8/8/8/8/K6k b - - 0 1".to_string());
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();

    let mut black_move = move_eval(
        id,
        1,
        Phase::Opening,
        MoveCategory::Best,
        TimeFlag::TimeTrouble,
    );
    black_move.player_color = Color::Black;
    black_move.win_prob_loss = 10.0;
    let mut white_move = move_eval(id, 2, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    white_move.player_color = Color::White;
    white_move.win_prob_loss = 1.0;
    let mut draft = analysis(id, 90.0, 80.0, vec![black_move, white_move]);
    draft.analysis.key_moments_count = 1;
    insert_analysis_draft(&db, &draft).unwrap();

    let stats = hub_stats(&db).unwrap();
    assert_eq!(
        stats.time.time_trouble_moves, 0,
        "флаг соперника не должен попадать в тайм-статистику игрока"
    );
    assert_eq!(stats.phases[0].moves, 1);
    assert!(
        (stats.openings[0].price - 1.0).abs() < 1e-9,
        "price={}",
        stats.openings[0].price
    );

    assert_eq!(backfill_summaries(&db).unwrap(), 1);
    let summary: String = db
        .with(|conn| {
            Ok(conn.query_row(
                "SELECT summary_text FROM game_analyses WHERE game_id = ?1",
                [id],
                |row| row.get(0),
            )?)
        })
        .unwrap();
    assert!(
        summary.contains("В цейтноте пройдено ходов: 0"),
        "{summary}"
    );
}

#[test]
fn fen_accuracy_backfill_keeps_sides_with_player_color() {
    let db = Db::in_memory().unwrap();
    let mut game = draft(
        "fen-accuracy",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    game.initial_fen = Some("8/8/8/8/8/8/8/K6k b - - 0 1".to_string());
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();

    // Чёрные (первый полуход) теряют почти всё преимущество, белые играют точно.
    let mut black_move = move_eval(id, 1, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    black_move.player_color = Color::Black;
    black_move.best_eval_cp = Some(50);
    black_move.win_prob_white = 80.0;
    let mut white_move = move_eval(id, 2, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    white_move.player_color = Color::White;
    white_move.best_eval_cp = Some(200);
    white_move.win_prob_white = 90.0;
    insert_analysis_draft(&db, &analysis(id, 1.0, 2.0, vec![black_move, white_move])).unwrap();

    assert_eq!(backfill_fen_accuracy(&db).unwrap(), 1);
    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert_eq!(stored.white_accuracy, Some(100.0));
    assert!(stored.black_accuracy.unwrap() < 100.0);
    assert_eq!(backfill_fen_accuracy(&db).unwrap(), 0);
}

#[test]
fn legacy_analysis_without_engine_version_stays_readable() {
    let db = Db::in_memory().unwrap();
    let game = draft(
        "legacy-version",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
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
    db.with(|conn| {
        conn.execute(
            "UPDATE game_analyses SET engine_version = NULL, config_key = '' WHERE game_id = ?1",
            [id],
        )?;
        Ok(())
    })
    .unwrap();

    let stored = get_analysis(&db, id).unwrap().unwrap();
    assert_eq!(
        stored.engine_version,
        crate::domain::analysis::DEFAULT_ENGINE_VERSION
    );
    // Без config_key анализ не выдаёт себя за проверенную конфигурацию.
    let stats = hub_stats(&db).unwrap();
    assert!(stats.config.is_none());
    assert_eq!(stats.kpis.analyzed_games, 1);
}

#[test]
fn migration_backfills_player_color_from_fen() {
    let db = Db::in_memory().unwrap();
    let mut game = draft(
        "fen-migrate",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    game.initial_fen = Some("8/8/8/8/8/8/8/K6k b - - 0 1".to_string());
    let id = db.with_tx(|tx| insert_game(tx, &game)).unwrap().unwrap();
    let mut black_move = move_eval(id, 1, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    black_move.fen_before = "8/8/8/8/8/8/8/K6k b - - 0 1".to_string();
    let white_move = move_eval(id, 2, Phase::Opening, MoveCategory::Best, TimeFlag::Normal);
    insert_analysis_draft(&db, &analysis(id, 90.0, 80.0, vec![black_move, white_move])).unwrap();

    db.with(|conn| {
        // Имитация базы версии 5: колонка есть, значения неверные.
        conn.execute("UPDATE move_evaluations SET player_color = 'white'", [])?;
        conn.pragma_update(None, "user_version", 5)?;
        migrations::run(conn)?;
        let first: String = conn.query_row(
            "SELECT player_color FROM move_evaluations WHERE game_id = ?1 AND ply = 1",
            [id],
            |row| row.get(0),
        )?;
        let second: String = conn.query_row(
            "SELECT player_color FROM move_evaluations WHERE game_id = ?1 AND ply = 2",
            [id],
            |row| row.get(0),
        )?;
        assert_eq!(first, "black");
        assert_eq!(second, "white");
        Ok(())
    })
    .unwrap();
}
