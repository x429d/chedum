use std::collections::HashMap;

use rusqlite::params_from_iter;

use crate::db::connection::Db;
use crate::domain::error::AppResult;

/// Analysis provenance labels for PGN export, one entry per game.
pub fn annotations_for_games(db: &Db, ids: &[i64]) -> AppResult<HashMap<i64, String>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    db.with(|conn| {
        let placeholders = vec!["?"; ids.len()].join(",");
        let sql = format!(
            "SELECT game_id, engine_name, depth, accuracy_method_version
             FROM game_analyses WHERE game_id IN ({placeholders})"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut out = HashMap::new();
        for row in rows {
            let (game_id, engine, depth, method) = row?;
            let label = if method == crate::domain::analysis::ACCURACY_METHOD_VERSION {
                "Lichess accuracy".to_string()
            } else {
                method
            };
            out.insert(
                game_id,
                format!("Chedum · {engine} · depth {depth} · {label}"),
            );
        }
        Ok(out)
    })
}
