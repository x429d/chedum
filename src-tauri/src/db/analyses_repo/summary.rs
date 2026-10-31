use rusqlite::{params, Connection, OptionalExtension};

use crate::analytics::{summary_from_analysis, SummaryContext};
use crate::db::connection::Db;
use crate::domain::analysis::GameAnalysis;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::UserColor;

use super::mapping::map_analysis;

const SUMMARY_VERSION_KEY: &str = "summary_text_version";
const SUMMARY_VERSION: &str = "single-source-v2";

/// Builds the summary text for a game, loading its fields and time-flag counts.
pub(super) fn summary_for_conn(
    conn: &Connection,
    game_id: i64,
    analysis: &GameAnalysis,
) -> AppResult<String> {
    let (white, black, result, user_color, eco, opening): (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn.query_row(
        "SELECT white_username, black_username, result, user_color, eco, opening_name
         FROM games WHERE id = ?1",
        params![game_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;
    let (time_trouble, impulsive) = user_time_flags(conn, game_id, &user_color)?;
    let context = SummaryContext {
        white: &white,
        black: &black,
        result: &result,
        user_color: UserColor::from_str_opt(&user_color).unwrap_or(UserColor::None),
        eco: eco.as_deref(),
        opening: opening.as_deref(),
    };
    Ok(summary_from_analysis(
        &context,
        analysis,
        time_trouble,
        impulsive,
    ))
}

fn user_time_flags(conn: &Connection, game_id: i64, user_color: &str) -> AppResult<(u32, u32)> {
    let Some(color) = UserColor::from_str_opt(user_color).and_then(UserColor::color) else {
        return Ok((0, 0));
    };
    conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN time_flag = 'time_trouble' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN time_flag = 'impulsive_error' THEN 1 ELSE 0 END), 0)
         FROM move_evaluations WHERE game_id = ?1 AND player_color = ?2",
        params![game_id, color.as_str()],
        |row| Ok((row.get::<_, i64>(0)? as u32, row.get::<_, i64>(1)? as u32)),
    )
    .map_err(AppError::from)
}

/// Rebuilds the stored summary text from the stored analysis metrics.
/// Needed once after switching to a single source of numbers; no engine required.
pub fn backfill_summaries(db: &Db) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [SUMMARY_VERSION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(SUMMARY_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
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
            params![SUMMARY_VERSION_KEY, SUMMARY_VERSION],
        )?;
        Ok(updated)
    })
}
