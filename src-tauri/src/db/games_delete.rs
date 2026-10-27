use rusqlite::params_from_iter;

use crate::db::connection::Db;
use crate::domain::error::AppResult;

pub fn delete_game(db: &Db, id: i64) -> AppResult<()> {
    delete_games(db, &[id]).map(|_| ())
}

/// Удаляет партии пакетно; связанные анализы и ходы удаляются каскадно.
pub fn delete_games(db: &Db, ids: &[i64]) -> AppResult<u32> {
    if ids.is_empty() {
        return Ok(0);
    }
    db.with_tx(|tx| {
        let placeholders = vec!["?"; ids.len()].join(",");
        let sql = format!("DELETE FROM games WHERE id IN ({placeholders})");
        let deleted = tx.execute(&sql, params_from_iter(ids.iter()))?;
        Ok(deleted as u32)
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "delete_tests.rs"]
mod tests;
