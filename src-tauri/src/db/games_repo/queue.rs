use rusqlite::types::Value;
use rusqlite::{params, params_from_iter};

use crate::db::connection::Db;
use crate::db::games_repo::list::filter_clause;
use crate::domain::error::AppResult;
use crate::domain::game::{GameFilter, PendingAnalysis};

pub fn pending_analysis(
    db: &Db,
    filter: &GameFilter,
    config_key: &str,
    target_depth: u32,
    target_multipv: u32,
) -> AppResult<PendingAnalysis> {
    db.with(|conn| {
        let mut filter = filter.clone();
        filter.analyzed_only = false;
        let (where_sql, values) = filter_clause(&filter);
        // A game counts as analyzed if it has the exact target configuration, a
        // record from an old build without config_key, or an analysis at least
        // as deep as the current settings. The last case keeps the queue from
        // downgrading an existing deeper analysis.
        let base = "FROM games g LEFT JOIN game_analyses a
                 ON a.game_id = g.id
                AND (a.config_key = ?1 OR a.config_key IS NULL OR a.config_key = ''
                     OR a.depth > ?2 OR (a.depth = ?2 AND a.multipv >= ?3))";
        let condition = if where_sql.is_empty() {
            "WHERE a.id IS NULL".to_string()
        } else {
            format!("{where_sql} AND a.id IS NULL")
        };
        let sql = format!(
            "SELECT g.id, g.total_plies {base} {condition} AND g.analysis_attempts < 3
             ORDER BY g.played_at DESC"
        );
        let mut params_vec = vec![
            Value::Text(config_key.to_string()),
            Value::Integer(i64::from(target_depth)),
            Value::Integer(i64::from(target_multipv)),
        ];
        params_vec.extend(values.iter().cloned());
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(params_vec.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
        })?;
        let mut ids = Vec::new();
        let mut positions = 0u64;
        for row in rows {
            let (id, plies) = row?;
            ids.push(id);
            positions += u64::from(plies.max(0) as u32) + 1;
        }
        let excluded_sql =
            format!("SELECT COUNT(*) {base} {condition} AND g.analysis_attempts >= 3");
        let excluded: i64 =
            conn.query_row(&excluded_sql, params_from_iter(params_vec.iter()), |row| {
                row.get(0)
            })?;
        Ok(PendingAnalysis {
            ids,
            positions,
            excluded: excluded as u32,
        })
    })
}

/// Records a failed analysis attempt; after the third, the game leaves the queue.
pub fn record_analysis_failure(db: &Db, id: i64, message: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET analysis_attempts = analysis_attempts + 1,
                last_analysis_error = ?2 WHERE id = ?1",
            params![id, message],
        )?;
        Ok(())
    })
}

/// A successful analysis clears past errors and returns the game to the queue
/// on the next configuration change.
pub fn reset_analysis_attempts(db: &Db, id: i64) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET analysis_attempts = 0, last_analysis_error = NULL
             WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    })
}
