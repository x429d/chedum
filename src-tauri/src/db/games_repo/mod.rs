use rusqlite::{params, params_from_iter, Connection, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{Game, GameDraft, TimeClass};

mod list;
mod mapping;
mod queue;

pub use list::list_games;
pub use queue::{pending_analysis, record_analysis_failure, reset_analysis_attempts};

use mapping::{map_game, GAME_COLUMNS};

pub fn insert_game(conn: &Connection, draft: &GameDraft) -> AppResult<Option<i64>> {
    let changed = conn.execute(
        "INSERT OR IGNORE INTO games (
            external_id, source, pgn, initial_fen, white_username, black_username, white_elo,
            black_elo, user_color, result, termination, time_control, time_class, eco,
            opening_name, opening_ply, opening_source, played_at, total_plies, notes, is_favorite
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
        params![
            draft.external_id,
            draft.source.as_str(),
            draft.pgn,
            draft.initial_fen,
            draft.white_username,
            draft.black_username,
            draft.white_elo,
            draft.black_elo,
            draft.user_color.as_str(),
            draft.result,
            draft.termination,
            draft.time_control,
            draft.time_class.map(TimeClass::as_str),
            draft.eco,
            draft.opening_name,
            draft.opening_ply,
            draft.opening_source,
            draft.played_at,
            draft.total_plies,
            draft.notes,
            i32::from(draft.is_favorite),
        ],
    )?;
    if changed == 0 {
        Ok(None)
    } else {
        Ok(Some(conn.last_insert_rowid()))
    }
}

pub fn count_games(db: &Db) -> AppResult<u32> {
    db.with(|conn| {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))?;
        Ok(count as u32)
    })
}

pub fn get_game(db: &Db, id: i64) -> AppResult<Game> {
    db.with(|conn| {
        conn.query_row(
            &format!("SELECT {GAME_COLUMNS} FROM games g WHERE g.id = ?1"),
            params![id],
            map_game,
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Партия #{id} не найдена")))
    })
}

pub fn set_favorite(db: &Db, id: i64, favorite: bool) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET is_favorite = ?1 WHERE id = ?2",
            params![i32::from(favorite), id],
        )?;
        Ok(())
    })
}

pub fn update_notes(db: &Db, id: i64, notes: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET notes = ?1 WHERE id = ?2",
            params![notes, id],
        )?;
        Ok(())
    })
}

pub fn games_by_ids(db: &Db, ids: &[i64]) -> AppResult<Vec<Game>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    db.with(|conn| {
        let placeholders = vec!["?"; ids.len()].join(",");
        let sql = format!(
            "SELECT {GAME_COLUMNS} FROM games g WHERE g.id IN ({placeholders}) ORDER BY g.played_at ASC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), map_game)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(games)
    })
}

pub fn all_games(db: &Db) -> AppResult<Vec<Game>> {
    db.with(|conn| {
        let sql = format!("SELECT {GAME_COLUMNS} FROM games g ORDER BY g.played_at ASC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_game)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(games)
    })
}


#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "../pagination_tests.rs"]
mod pagination_tests;
