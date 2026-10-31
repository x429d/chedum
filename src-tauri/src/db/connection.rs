use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, Transaction};

use crate::db::migrations;
use crate::domain::error::{AppError, AppResult};

/// Thread-safe SQLite connection. All operations are serialized by a mutex,
/// which for a local desktop app is faster and simpler than a pool.
#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        configure(&conn)?;
        migrations::backup_before_upgrade(&conn, path)?;
        migrations::run(&conn)?;
        Ok(Db {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        configure(&conn)?;
        migrations::run(&conn)?;
        Ok(Db {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<T>(&self, action: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let guard = self.lock()?;
        action(&guard)
    }

    pub fn with_tx<T>(
        &self,
        action: impl FnOnce(&Transaction<'_>) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut guard = self.lock()?;
        let tx = guard.transaction()?;
        let value = action(&tx)?;
        tx.commit()?;
        Ok(value)
    }

    /// Runs blocking SQLite work on the blocking pool so async Tauri commands
    /// do not occupy a runtime worker during large queries or imports.
    pub async fn run<T, F>(&self, action: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&Db) -> AppResult<T> + Send + 'static,
    {
        let db = self.clone();
        tauri::async_runtime::spawn_blocking(move || action(&db))
            .await
            .map_err(|err| AppError::Internal(format!("Database task failed: {err}")))?
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.inner
            .lock()
            .map_err(|_| AppError::Internal("SQLite connection is locked".to_string()))
    }
}

fn configure(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;",
    )?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_all_tables() {
        let db = Db::in_memory().unwrap();
        db.with(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('games','game_analyses','move_evaluations','synced_archives','app_settings','accounts')",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 6);
            let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
            assert!(mode == "memory" || mode == "wal");
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn foreign_keys_are_enabled() {
        let db = Db::in_memory().unwrap();
        db.with(|conn| {
            let value: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
            assert_eq!(value, 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn creates_a_backup_before_upgrading_a_file_database() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("chedum-backup-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chedum.db");
        {
            let db = Db::open(&path).unwrap();
            db.with(|conn| {
                conn.pragma_update(None, "user_version", 1)?;
                Ok(())
            })
            .unwrap();
        }
        let db = Db::open(&path).unwrap();
        db.with(|conn| {
            let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            assert_eq!(version, migrations::CURRENT_SCHEMA_VERSION);
            Ok(())
        })
        .unwrap();
        assert!(dir.join("chedum.pre-v1.db").exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
