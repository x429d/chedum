use rusqlite::{params, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::analysis::{GameAnalysis, MoveEvaluation};
use crate::domain::error::AppResult;

mod annotations;
mod backfill;
pub mod checkpoints;
mod insert;
mod mapping;
mod summary;

use mapping::{map_analysis, map_move};
use summary::summary_for_conn;

pub use annotations::annotations_for_games;
pub use backfill::{
    backfill_config_keys, backfill_fen_accuracy, backfill_key_moments, backfill_lichess_accuracy,
};
pub use insert::{analysis_config_key, insert_analysis_conn, insert_analysis_draft};
pub use summary::backfill_summaries;

pub fn get_analysis(db: &Db, game_id: i64) -> AppResult<Option<GameAnalysis>> {
    db.with(|conn| {
        let analysis = conn
            .query_row(
                "SELECT * FROM game_analyses WHERE game_id = ?1",
                params![game_id],
                map_analysis,
            )
            .optional()?;
        match analysis {
            Some(mut analysis) => {
                // The summary text is always assembled from the same fields as
                // the cards, so old rows cannot show a different accuracy number.
                analysis.summary_text = summary_for_conn(conn, game_id, &analysis)?;
                Ok(Some(analysis))
            }
            None => Ok(None),
        }
    })
}

pub fn save_move_comment(db: &Db, game_id: i64, ply: i32, comment: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE move_evaluations SET user_comment = ?1 WHERE game_id = ?2 AND ply = ?3",
            params![comment, game_id, ply],
        )?;
        Ok(())
    })
}

pub fn get_moves(db: &Db, game_id: i64) -> AppResult<Vec<MoveEvaluation>> {
    db.with(|conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC")?;
        let rows = stmt.query_map(params![game_id], map_move)?;
        let mut moves = Vec::new();
        for row in rows {
            moves.push(row?);
        }
        Ok(moves)
    })
}

pub fn has_analysis(db: &Db, game_id: i64) -> AppResult<bool> {
    db.with(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM game_analyses WHERE game_id = ?1",
            params![game_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    })
}
