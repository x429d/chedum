use chrono::Utc;
use rusqlite::{params, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::error::AppResult;

/// Identity of a checkpoint set. A change of depth, MultiPV, or engine makes
/// old checkpoints irrelevant.
pub struct CheckpointKey<'a> {
    pub game_id: i64,
    pub depth: u32,
    pub multipv: u32,
    pub engine_name: &'a str,
    pub engine_version: &'a str,
}

const SELECT_BLOCK: &str = "SELECT evaluations_json FROM analysis_checkpoints
     WHERE game_id = ?1 AND block_index = ?2 AND depth = ?3 AND multipv = ?4
       AND engine_name = ?5 AND engine_version = ?6";

pub fn load_block(
    db: &Db,
    key: &CheckpointKey<'_>,
    block_index: usize,
) -> AppResult<Option<String>> {
    db.with(|conn| {
        let json: Option<String> = conn
            .query_row(
                SELECT_BLOCK,
                params![
                    key.game_id,
                    block_index as i64,
                    key.depth,
                    key.multipv,
                    key.engine_name,
                    key.engine_version
                ],
                |row| row.get(0),
            )
            .optional()?;
        Ok(json)
    })
}

pub fn store_block(
    db: &Db,
    key: &CheckpointKey<'_>,
    block_index: usize,
    evaluations_json: &str,
) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "INSERT INTO analysis_checkpoints (
                game_id, block_index, depth, multipv, engine_name, engine_version,
                evaluations_json, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(game_id, block_index, depth, multipv, engine_name, engine_version)
             DO UPDATE SET evaluations_json = excluded.evaluations_json,
                           updated_at = excluded.updated_at",
            params![
                key.game_id,
                block_index as i64,
                key.depth,
                key.multipv,
                key.engine_name,
                key.engine_version,
                evaluations_json,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    })
}

/// Removes checkpoints that belong to a different engine, depth, or MultiPV.
pub fn clear_stale(db: &Db, key: &CheckpointKey<'_>) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_checkpoints
             WHERE game_id = ?1 AND NOT (
                depth = ?2 AND multipv = ?3 AND engine_name = ?4 AND engine_version = ?5
             )",
            params![
                key.game_id,
                key.depth,
                key.multipv,
                key.engine_name,
                key.engine_version
            ],
        )?;
        Ok(())
    })
}

pub fn clear_game(db: &Db, game_id: i64) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "DELETE FROM analysis_checkpoints WHERE game_id = ?1",
            params![game_id],
        )?;
        Ok(())
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn key() -> CheckpointKey<'static> {
        CheckpointKey {
            game_id: 1,
            depth: 14,
            multipv: 2,
            engine_name: "Stockfish 19",
            engine_version: "19",
        }
    }

    #[test]
    fn checkpoint_roundtrip_and_stale_cleanup() {
        let db = Db::in_memory().unwrap();
        db.with(|conn| {
            conn.execute(
                "INSERT INTO games (external_id, source, pgn, white_username, black_username,
                    user_color, result, played_at, total_plies)
                 VALUES ('g1', 'manual', '', 'a', 'b', 'white', '1-0', 1, 1)",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        let key = key();
        store_block(&db, &key, 0, "[{\"fen\":\"x\"}]").unwrap();
        assert_eq!(
            load_block(&db, &key, 0).unwrap().as_deref(),
            Some("[{\"fen\":\"x\"}]")
        );
        let other = CheckpointKey { depth: 18, ..key };
        clear_stale(&db, &other).unwrap();
        assert!(load_block(&db, &key, 0).unwrap().is_none());
        store_block(&db, &other, 0, "[]").unwrap();
        clear_game(&db, 1).unwrap();
        assert!(load_block(&db, &other, 0).unwrap().is_none());
    }
}
