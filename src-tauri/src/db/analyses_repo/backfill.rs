use rusqlite::{params, Connection, OptionalExtension};

use crate::analytics::{
    game_accuracy, performance_elo, phase_accuracy, win_prob_cp, win_prob_mate,
};
use crate::db::connection::Db;
use crate::domain::analysis::{MoveEvaluation, Phase};
use crate::domain::error::AppResult;
use crate::domain::game::Color;

use super::mapping::{map_analysis, map_move};
use super::summary::summary_for_conn;

const ACCURACY_ALGO_KEY: &str = "accuracy_algo";
const KEY_MOMENTS_VERSION_KEY: &str = "key_moments_version";
const KEY_MOMENTS_VERSION: &str = "errors-gap3-v2";
const FEN_ACCURACY_KEY: &str = "fen_accuracy_version";
const FEN_ACCURACY_VERSION: &str = "player-color-v1";

/// Repairs rows left behind by older builds: when the binary that wrote a row
/// knew nothing about the version and config-key columns, `engine_version` and
/// `config_key` stay empty. The migration does not catch those rows — it runs
/// only once — so we repair them on every startup.
pub fn backfill_config_keys(db: &Db) -> AppResult<usize> {
    db.with(|conn| {
        let updated = conn.execute(
            "UPDATE game_analyses SET
                engine_version = COALESCE(NULLIF(engine_version, ''), ?1),
                config_key = engine_name
                    || '|' || COALESCE(NULLIF(engine_version, ''), ?1)
                    || '|' || depth
                    || '|' || multipv
                    || '|' || COALESCE(NULLIF(classifier_version, ''), ?2)
                    || '|' || COALESCE(NULLIF(accuracy_method_version, ''), ?3)
             WHERE config_key IS NULL OR config_key = ''
                OR engine_version IS NULL OR engine_version = ''",
            params![
                crate::domain::analysis::DEFAULT_ENGINE_VERSION,
                crate::domain::analysis::CLASSIFIER_VERSION,
                crate::domain::analysis::ACCURACY_METHOD_VERSION
            ],
        )?;
        Ok(updated)
    })
}

/// Recomputes key moments from stored evaluations. No engine needed: after the
/// selection algorithm changes, re-examining the moves is enough.
pub fn backfill_key_moments(db: &Db) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [KEY_MOMENTS_VERSION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(KEY_MOMENTS_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            let user_color: String = tx.query_row(
                "SELECT user_color FROM games WHERE id = ?1",
                [game_id],
                |row| row.get(0),
            )?;
            let moves = load_moves_conn(tx, game_id)?;
            let selected = crate::analytics::select_key_moments(
                &moves,
                Color::from_str_opt(&user_color),
                crate::analytics::moments::player_from_fen,
            );
            for (index, mv) in moves.iter().enumerate() {
                tx.execute(
                    "UPDATE move_evaluations SET is_key_moment = ?2
                     WHERE game_id = ?1 AND ply = ?3",
                    params![game_id, i32::from(selected.contains(&index)), mv.ply],
                )?;
            }
            tx.execute(
                "UPDATE game_analyses SET key_moments_count = ?2 WHERE game_id = ?1",
                params![game_id, selected.len() as i32],
            )?;
            let analysis = tx.query_row(
                "SELECT * FROM game_analyses WHERE game_id = ?1",
                params![game_id],
                map_analysis,
            )?;
            let summary = summary_for_conn(tx, game_id, &analysis)?;
            tx.execute(
                "UPDATE game_analyses SET summary_text = ?2 WHERE game_id = ?1",
                params![game_id, summary],
            )?;
            updated += 1;
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![KEY_MOMENTS_VERSION_KEY, KEY_MOMENTS_VERSION],
        )?;
        Ok(updated)
    })
}

fn load_moves_conn(conn: &Connection, game_id: i64) -> AppResult<Vec<MoveEvaluation>> {
    let mut stmt =
        conn.prepare("SELECT * FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC")?;
    let rows = stmt.query_map(params![game_id], map_move)?;
    let mut moves = Vec::new();
    for row in rows {
        moves.push(row?);
    }
    Ok(moves)
}

/// Stored move used for recomputation: White's win% after it, evaluation before
/// it, phase, and player.
type StoredMoveRow = (f64, Option<i32>, Option<i32>, Phase, Color);

/// Recomputes accuracy with the Lichess method from already stored evaluations.
/// Needed once after an algorithm change: no engine or re-analysis required.
/// `force` recomputes even if the algorithm version is already recorded (for
/// example, after demo games are re-imported).
pub fn backfill_lichess_accuracy(db: &Db, force: bool) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [ACCURACY_ALGO_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if !force && current.as_deref() == Some(crate::domain::analysis::ACCURACY_METHOD_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            if recompute_accuracy(tx, game_id)? {
                updated += 1;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![
                ACCURACY_ALGO_KEY,
                crate::domain::analysis::ACCURACY_METHOD_VERSION
            ],
        )?;
        Ok(updated)
    })
}

/// Recomputes accuracy for games with a non-standard starting position: before
/// `player_color` existed, recomputation inferred the player from ply parity
/// and could swap the sides. No engine needed.
pub fn backfill_fen_accuracy(db: &Db) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [FEN_ACCURACY_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(FEN_ACCURACY_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare(
                "SELECT a.game_id FROM game_analyses a JOIN games g ON g.id = a.game_id
                 WHERE g.initial_fen IS NOT NULL ORDER BY a.game_id",
            )?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            if recompute_accuracy(tx, game_id)? {
                updated += 1;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![FEN_ACCURACY_KEY, FEN_ACCURACY_VERSION],
        )?;
        Ok(updated)
    })
}

fn recompute_accuracy(conn: &Connection, game_id: i64) -> AppResult<bool> {
    let (white_acpl, black_acpl, white_elo, black_elo) = conn.query_row(
        "SELECT a.white_acpl, a.black_acpl, g.white_elo, g.black_elo
         FROM game_analyses a JOIN games g ON g.id = a.game_id WHERE a.game_id = ?1",
        [game_id],
        |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?,
                row.get::<_, Option<f64>>(1)?,
                row.get::<_, Option<i32>>(2)?,
                row.get::<_, Option<i32>>(3)?,
            ))
        },
    )?;
    let rows: Vec<StoredMoveRow> = {
        let mut stmt = conn.prepare(
            "SELECT win_prob_white, best_eval_cp, best_eval_mate, phase, player_color
             FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC",
        )?;
        let mapped = stmt.query_map([game_id], |row| {
            let phase: String = row.get(3)?;
            let player_color: String = row.get(4)?;
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                Phase::from_str_opt(&phase).unwrap_or(Phase::Middlegame),
                Color::from_str_opt(&player_color).unwrap_or(Color::White),
            ))
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()?
    };
    if rows.is_empty() {
        return Ok(false);
    }

    let mut samples = Vec::with_capacity(rows.len());
    let mut phase_samples: [Vec<(f64, f64)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut phase_first: [Option<Color>; 3] = [None, None, None];
    let mut first_color = Color::White;
    for (index, (win_white, cp, mate, phase, player)) in rows.iter().enumerate() {
        let player = *player;
        let before_white = score_win_prob(*cp, *mate);
        let (before, after) = if matches!(player, Color::White) {
            (before_white, *win_white)
        } else {
            (100.0 - before_white, 100.0 - *win_white)
        };
        let sample = (before, after);
        samples.push(sample);
        if index == 0 {
            first_color = player;
        }
        let slot = match phase {
            Phase::Opening => 0,
            Phase::Middlegame => 1,
            Phase::Endgame => 2,
        };
        if phase_first[slot].is_none() {
            phase_first[slot] = Some(player);
        }
        phase_samples[slot].push(sample);
    }

    let [white_accuracy, black_accuracy] = game_accuracy(&samples, first_color);
    let [white_phases, black_phases] = phase_accuracy(&phase_samples, &phase_first);
    conn.execute(
        "UPDATE game_analyses SET
            white_accuracy = ?2, black_accuracy = ?3,
            white_opening_acc = ?4, white_middlegame_acc = ?5, white_endgame_acc = ?6,
            black_opening_acc = ?7, black_middlegame_acc = ?8, black_endgame_acc = ?9,
            white_perf_elo = ?10, black_perf_elo = ?11
         WHERE game_id = ?1",
        params![
            game_id,
            white_accuracy,
            black_accuracy,
            white_phases[0],
            white_phases[1],
            white_phases[2],
            black_phases[0],
            black_phases[1],
            black_phases[2],
            performance_elo(white_elo, white_accuracy, white_acpl),
            performance_elo(black_elo, black_accuracy, black_acpl),
        ],
    )?;
    // The summary lives alongside accuracy and must be recomputed with it.
    let analysis = conn.query_row(
        "SELECT * FROM game_analyses WHERE game_id = ?1",
        params![game_id],
        map_analysis,
    )?;
    let summary = summary_for_conn(conn, game_id, &analysis)?;
    conn.execute(
        "UPDATE game_analyses SET summary_text = ?2 WHERE game_id = ?1",
        params![game_id, summary],
    )?;
    Ok(true)
}

fn score_win_prob(cp: Option<i32>, mate: Option<i32>) -> f64 {
    match mate {
        Some(mate) => win_prob_mate(mate),
        None => win_prob_cp(f64::from(cp.unwrap_or(0))),
    }
}
