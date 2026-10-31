use rusqlite::{params, params_from_iter};
use rusqlite::types::Value;

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
        // Партия разобрана, если есть точная целевая конфигурация, запись
        // старой сборки без config_key или анализ не мельче текущих настроек.
        // Последнее не даёт очереди понижать уже более глубокий разбор.
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
        let excluded_sql = format!(
            "SELECT COUNT(*) {base} {condition} AND g.analysis_attempts >= 3"
        );
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

/// Фиксирует неудачную попытку анализа; после третьей партия выходит из очереди.
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

/// Успешный анализ снимает прошлые ошибки и возвращает партию в очередь
/// при следующей смене конфигурации.
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
