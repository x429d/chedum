use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::schema::V1;
use crate::domain::error::AppResult;

pub const CURRENT_SCHEMA_VERSION: i64 = 10;

pub fn current_version(conn: &Connection) -> AppResult<i64> {
    Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

/// Creates a consistent copy of the database before a destructive upgrade.
/// `VACUUM INTO` includes committed WAL content, unlike a raw file copy.
pub fn backup_before_upgrade(conn: &Connection, path: &Path) -> AppResult<Option<PathBuf>> {
    let version = current_version(conn)?;
    if version == 0 || version >= CURRENT_SCHEMA_VERSION {
        return Ok(None);
    }
    let backup = path.with_file_name(format!("chenal.pre-v{version}.db"));
    if backup.exists() {
        return Ok(Some(backup));
    }
    let target = backup.to_string_lossy().to_string();
    conn.execute("VACUUM INTO ?1", params![target])?;
    Ok(Some(backup))
}

fn step<F>(conn: &Connection, target: i64, action: F) -> AppResult<()>
where
    F: FnOnce(&Connection) -> AppResult<()>,
{
    conn.execute_batch("BEGIN IMMEDIATE")?;
    match action(conn) {
        Ok(()) => {
            conn.pragma_update(None, "user_version", target)?;
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

pub fn run(conn: &Connection) -> AppResult<()> {
    let version = current_version(conn)?;
    if version < 1 {
        step(conn, 1, |conn| {
            conn.execute_batch(V1)?;
            Ok(())
        })?;
    }
    if version < 2 {
        step(conn, 2, |conn| {
            add_column_if_missing(conn, "games", "opening_ply", "INTEGER")?;
            add_column_if_missing(conn, "games", "opening_source", "TEXT")?;
            Ok(())
        })?;
    }
    if version < 3 {
        step(conn, 3, |conn| {
            add_column_if_missing(
                conn,
                "move_evaluations",
                "is_key_moment",
                "INTEGER NOT NULL DEFAULT 0",
            )
        })?;
    }
    if version < 4 {
        step(conn, 4, |conn| {
            add_column_if_missing(conn, "game_analyses", "engine_version", "TEXT")?;
            add_column_if_missing(
                conn,
                "game_analyses",
                "classifier_version",
                "TEXT NOT NULL DEFAULT 'classifier-v1'",
            )?;
            add_column_if_missing(
                conn,
                "game_analyses",
                "accuracy_method_version",
                "TEXT NOT NULL DEFAULT 'lichess-v1'",
            )?;
            add_column_if_missing(
                conn,
                "game_analyses",
                "config_key",
                "TEXT NOT NULL DEFAULT ''",
            )?;
            conn.execute(
                "UPDATE game_analyses SET engine_version = '19'
                 WHERE engine_version IS NULL OR engine_version = ''",
                [],
            )?;
            conn.execute(
                "UPDATE game_analyses SET config_key =
                    engine_name || '|' || engine_version || '|' || depth || '|' || multipv
                    || '|' || classifier_version || '|' || accuracy_method_version",
                [],
            )?;
            Ok(())
        })?;
    }
    if version < 5 {
        step(conn, 5, |conn| {
            add_column_if_missing(
                conn,
                "games",
                "analysis_attempts",
                "INTEGER NOT NULL DEFAULT 0",
            )?;
            add_column_if_missing(conn, "games", "last_analysis_error", "TEXT")?;
            // New installations get auto-analysis enabled; existing databases
            // keep the previous behavior until the user explicitly enables it.
            conn.execute(
                "INSERT INTO app_settings (key, value)
                 SELECT 'auto_analyze', 'false'
                 WHERE NOT EXISTS (SELECT 1 FROM app_settings WHERE key = 'auto_analyze')
                   AND EXISTS (SELECT 1 FROM games)",
                [],
            )?;
            Ok(())
        })?;
    }
    if version < 6 {
        step(conn, 6, |conn| {
            add_column_if_missing(
                conn,
                "move_evaluations",
                "player_color",
                "TEXT NOT NULL DEFAULT 'white'",
            )?;
            // Derive the side from the FEN before the move. Ply parity is wrong
            // for games started from a non-standard position.
            conn.execute(
                "UPDATE move_evaluations SET player_color = CASE
                    WHEN substr(fen_before, instr(fen_before, ' ') + 1, 1) = 'b' THEN 'black'
                    ELSE 'white' END",
                [],
            )?;
            Ok(())
        })?;
    }
    if version < 7 {
        step(conn, 7, |conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER PRIMARY KEY,
                    platform TEXT NOT NULL,
                    username TEXT NOT NULL COLLATE NOCASE,
                    created_at TEXT NOT NULL,
                    last_synced_at TEXT,
                    UNIQUE(platform, username)
                );",
            )?;
            add_column_if_missing(
                conn,
                "games",
                "account_id",
                "INTEGER REFERENCES accounts(id) ON DELETE SET NULL",
            )?;
            conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_games_account ON games(account_id);",
            )?;
            backfill_legacy_account(conn)?;
            Ok(())
        })?;
    }
    if version < 8 {
        step(conn, 8, |conn| {
            // Engine games are not part of personal analytics and should not
            // occupy library space. Remove already synchronized ones.
            conn.execute(
                "DELETE FROM games
                 WHERE lower(white_username) LIKE 'lichess ai level %'
                    OR lower(black_username) LIKE 'lichess ai level %'
                    OR lower(white_username) = 'computer'
                    OR lower(black_username) = 'computer'
                    OR pgn LIKE '%[WhiteTitle \"BOT\"]%'
                    OR pgn LIKE '%[BlackTitle \"BOT\"]%'",
                [],
            )?;
            Ok(())
        })?;
    }
    if version < 9 {
        step(conn, 9, |conn| {
            add_column_if_missing(conn, "move_evaluations", "explanation_params_json", "TEXT")
        })?;
    }
    if version < 10 {
        step(conn, 10, |conn| {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS analysis_checkpoints (
                    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
                    block_index INTEGER NOT NULL,
                    depth INTEGER NOT NULL,
                    multipv INTEGER NOT NULL,
                    engine_name TEXT NOT NULL,
                    engine_version TEXT NOT NULL,
                    evaluations_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    PRIMARY KEY (game_id, block_index, depth, multipv, engine_name, engine_version)
                );",
            )?;
            Ok(())
        })?;
    }
    Ok(())
}

/// Migrates the old single profile from settings into the accounts table:
/// otherwise games would lose their player link after the upgrade.
fn backfill_legacy_account(conn: &Connection) -> AppResult<()> {
    let legacy: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings
             WHERE key = 'player_username' AND trim(value) <> ''",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let Some(username) = legacy else {
        return Ok(());
    };
    let existing: Option<i64> = conn
        .query_row("SELECT id FROM accounts LIMIT 1", [], |row| row.get(0))
        .optional()?;
    if existing.is_some() {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO accounts (platform, username, created_at) VALUES ('chesscom', ?1, ?2)",
        params![username, Utc::now().to_rfc3339()],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "UPDATE games SET account_id = ?1
         WHERE account_id IS NULL AND source = 'chesscom'
           AND user_color IN ('white', 'black')
           AND (white_username = ?2 COLLATE NOCASE OR black_username = ?2 COLLATE NOCASE)",
        params![id, username],
    )?;
    Ok(())
}

/// Idempotent column addition: a rerun after a failure does not crash.
pub fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> AppResult<()> {
    if column_exists(conn, table, column)? {
        return Ok(());
    }
    conn.execute_batch(&format!(
        "ALTER TABLE {table} ADD COLUMN {column} {definition}"
    ))?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}
