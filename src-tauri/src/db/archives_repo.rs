use rusqlite::{params, Connection, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::error::AppResult;

#[derive(Debug, Clone)]
pub struct ArchiveRecord {
    pub username: String,
    pub archive_url: String,
    pub year: i32,
    pub month: i32,
    pub is_fully_synced: bool,
    pub last_synced_at: String,
}

pub fn list_for_user(db: &Db, username: &str) -> AppResult<Vec<ArchiveRecord>> {
    db.with(|conn| list_for_user_conn(conn, username))
}

pub fn list_for_user_conn(conn: &Connection, username: &str) -> AppResult<Vec<ArchiveRecord>> {
    let mut stmt = conn.prepare(
        "SELECT username, archive_url, year, month, is_fully_synced, last_synced_at
         FROM synced_archives WHERE username = ?1 ORDER BY year, month",
    )?;
    let rows = stmt.query_map(params![username], |row| {
        Ok(ArchiveRecord {
            username: row.get(0)?,
            archive_url: row.get(1)?,
            year: row.get(2)?,
            month: row.get(3)?,
            is_fully_synced: row.get::<_, i64>(4)? != 0,
            last_synced_at: row.get(5)?,
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

pub fn find(db: &Db, username: &str, archive_url: &str) -> AppResult<Option<ArchiveRecord>> {
    db.with(|conn| {
        conn.query_row(
            "SELECT username, archive_url, year, month, is_fully_synced, last_synced_at
             FROM synced_archives WHERE username = ?1 AND archive_url = ?2",
            params![username, archive_url],
            |row| {
                Ok(ArchiveRecord {
                    username: row.get(0)?,
                    archive_url: row.get(1)?,
                    year: row.get(2)?,
                    month: row.get(3)?,
                    is_fully_synced: row.get::<_, i64>(4)? != 0,
                    last_synced_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    })
}

pub fn upsert(
    db: &Db,
    username: &str,
    archive_url: &str,
    year: i32,
    month: i32,
    fully_synced: bool,
    synced_at: &str,
) -> AppResult<()> {
    db.with(|conn| {
        upsert_conn(
            conn,
            username,
            archive_url,
            year,
            month,
            fully_synced,
            synced_at,
        )
    })
}

pub fn upsert_conn(
    conn: &Connection,
    username: &str,
    archive_url: &str,
    year: i32,
    month: i32,
    fully_synced: bool,
    synced_at: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO synced_archives (username, archive_url, year, month, is_fully_synced, last_synced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(archive_url) DO UPDATE SET
            username = excluded.username,
            year = excluded.year,
            month = excluded.month,
            is_fully_synced = excluded.is_fully_synced,
            last_synced_at = excluded.last_synced_at",
        params![username, archive_url, year, month, i32::from(fully_synced), synced_at],
    )?;
    Ok(())
}
