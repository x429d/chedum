use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::error::AppResult;

const V1: &str = r#"
CREATE TABLE IF NOT EXISTS synced_archives (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    archive_url TEXT UNIQUE NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    is_fully_synced INTEGER NOT NULL DEFAULT 0,
    last_synced_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY,
    external_id TEXT UNIQUE NOT NULL,
    source TEXT NOT NULL,
    pgn TEXT NOT NULL,
    initial_fen TEXT,
    white_username TEXT NOT NULL,
    black_username TEXT NOT NULL,
    white_elo INTEGER,
    black_elo INTEGER,
    user_color TEXT NOT NULL DEFAULT 'none',
    result TEXT NOT NULL,
    termination TEXT,
    time_control TEXT,
    time_class TEXT,
    eco TEXT,
    opening_name TEXT,
    played_at INTEGER NOT NULL,
    total_plies INTEGER NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    is_favorite INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS game_analyses (
    id INTEGER PRIMARY KEY,
    game_id INTEGER UNIQUE NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    engine_name TEXT NOT NULL,
    depth INTEGER NOT NULL,
    multipv INTEGER NOT NULL,
    analyzed_at TEXT NOT NULL,
    white_accuracy REAL,
    black_accuracy REAL,
    white_acpl REAL,
    black_acpl REAL,
    white_perf_elo INTEGER,
    black_perf_elo INTEGER,
    white_opening_acc REAL,
    white_middlegame_acc REAL,
    white_endgame_acc REAL,
    black_opening_acc REAL,
    black_middlegame_acc REAL,
    black_endgame_acc REAL,
    counts_json TEXT NOT NULL,
    summary_text TEXT NOT NULL,
    key_moments_count INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS move_evaluations (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    ply INTEGER NOT NULL,
    phase TEXT NOT NULL,
    fen_before TEXT NOT NULL,
    fen_after TEXT NOT NULL,
    played_uci TEXT NOT NULL,
    played_san TEXT NOT NULL,
    best_uci TEXT,
    best_san TEXT,
    eval_cp INTEGER,
    eval_mate INTEGER,
    best_eval_cp INTEGER,
    best_eval_mate INTEGER,
    win_prob_white REAL NOT NULL,
    win_prob_loss REAL NOT NULL,
    move_accuracy REAL NOT NULL,
    classification TEXT NOT NULL,
    is_critical INTEGER NOT NULL DEFAULT 0,
    material_diff_white INTEGER NOT NULL,
    clock_remaining_sec REAL,
    time_spent_sec REAL,
    time_flag TEXT NOT NULL DEFAULT 'normal',
    explanation_tag TEXT,
    explanation_text TEXT,
    pv_lines_json TEXT NOT NULL,
    user_comment TEXT NOT NULL DEFAULT '',
    UNIQUE(game_id, ply)
);

CREATE INDEX IF NOT EXISTS idx_games_played_at ON games(played_at);
CREATE INDEX IF NOT EXISTS idx_games_white ON games(white_username);
CREATE INDEX IF NOT EXISTS idx_games_black ON games(black_username);
CREATE INDEX IF NOT EXISTS idx_games_time_class ON games(time_class);
CREATE INDEX IF NOT EXISTS idx_games_eco ON games(eco);
CREATE INDEX IF NOT EXISTS idx_moves_game ON move_evaluations(game_id, ply);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub fn run(conn: &Connection) -> AppResult<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch(V1)?;
        conn.pragma_update(None, "user_version", 1)?;
    }
    if version < 2 {
        add_column_if_missing(conn, "games", "opening_ply", "INTEGER")?;
        add_column_if_missing(conn, "games", "opening_source", "TEXT")?;
        conn.pragma_update(None, "user_version", 2)?;
    }
    if version < 3 {
        add_column_if_missing(
            conn,
            "move_evaluations",
            "is_key_moment",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        conn.pragma_update(None, "user_version", 3)?;
    }
    if version < 4 {
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
        conn.pragma_update(None, "user_version", 4)?;
    }
    if version < 5 {
        add_column_if_missing(
            conn,
            "games",
            "analysis_attempts",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        add_column_if_missing(conn, "games", "last_analysis_error", "TEXT")?;
        // Новые установки получают автоанализ включённым (значение по
        // умолчанию в AppSettings). У существующей базы, где пользователь
        // ещё не выбирал, сохраняем прежнее поведение — выключено.
        conn.execute(
            "INSERT INTO app_settings (key, value)
             SELECT 'auto_analyze', 'false'
             WHERE NOT EXISTS (SELECT 1 FROM app_settings WHERE key = 'auto_analyze')
               AND EXISTS (SELECT 1 FROM games)",
            [],
        )?;
        conn.pragma_update(None, "user_version", 5)?;
    }
    if version < 6 {
        add_column_if_missing(
            conn,
            "move_evaluations",
            "player_color",
            "TEXT NOT NULL DEFAULT 'white'",
        )?;
        // Цвет берётся из FEN перед ходом: чётность полухода ошибается
        // в партиях, начатых с нестандартной позиции.
        conn.execute(
            "UPDATE move_evaluations SET player_color = CASE
                WHEN substr(fen_before, instr(fen_before, ' ') + 1, 1) = 'b' THEN 'black'
                ELSE 'white' END",
            [],
        )?;
        conn.pragma_update(None, "user_version", 6)?;
    }
    if version < 7 {
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
        conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_games_account ON games(account_id);")?;
        backfill_legacy_account(conn)?;
        conn.pragma_update(None, "user_version", 7)?;
    }
    if version < 8 {
        // Партии с компьютером не участвуют в личной аналитике и не должны
        // занимать место в библиотеке. Удаляем уже синхронизированные.
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
        conn.pragma_update(None, "user_version", 8)?;
    }
    Ok(())
}

/// Переносит старый одиночный профиль из настроек в таблицу аккаунтов:
/// иначе после обновления партии потеряли бы привязку к игроку.
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

/// Идемпотентное добавление колонки: повторный запуск после сбоя не падает.
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
