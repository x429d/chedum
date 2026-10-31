use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::account::{Account, AccountPlatform};
use crate::domain::error::{AppError, AppResult};

const SELECT_ACCOUNT: &str = "SELECT a.id, a.platform, a.username, a.created_at, a.last_synced_at,
        (SELECT COUNT(*) FROM games g WHERE g.account_id = a.id) AS games,
        (SELECT COUNT(*) FROM games g JOIN game_analyses ga ON ga.game_id = g.id
         WHERE g.account_id = a.id) AS analyzed
     FROM accounts a";

fn map_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<Account> {
    let platform = row.get::<_, String>("platform")?;
    Ok(Account {
        id: row.get("id")?,
        platform: AccountPlatform::from_str_opt(&platform).unwrap_or(AccountPlatform::Chesscom),
        username: row.get("username")?,
        created_at: row.get("created_at")?,
        last_synced_at: row.get("last_synced_at")?,
        games: row.get::<_, i64>("games")? as u32,
        analyzed: row.get::<_, i64>("analyzed")? as u32,
    })
}

pub fn list(db: &Db) -> AppResult<Vec<Account>> {
    db.with(|conn| {
        let sql =
            format!("{SELECT_ACCOUNT} ORDER BY a.platform ASC, a.username COLLATE NOCASE ASC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_account)?;
        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(row?);
        }
        Ok(accounts)
    })
}

pub fn get(db: &Db, id: i64) -> AppResult<Account> {
    db.with(|conn| {
        conn.query_row(
            &format!("{SELECT_ACCOUNT} WHERE a.id = ?1"),
            [id],
            map_account,
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Аккаунт #{id} не найден")))
    })
}

pub fn find_by_username(
    db: &Db,
    platform: AccountPlatform,
    username: &str,
) -> AppResult<Option<Account>> {
    db.with(|conn| {
        conn.query_row(
            &format!("{SELECT_ACCOUNT} WHERE a.platform = ?1 AND a.username = ?2 COLLATE NOCASE"),
            params![platform.as_str(), username.trim()],
            map_account,
        )
        .optional()
        .map_err(Into::into)
    })
}

pub fn count(db: &Db) -> AppResult<u32> {
    db.with(|conn| {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        Ok(count as u32)
    })
}

pub fn create(db: &Db, platform: AccountPlatform, username: &str) -> AppResult<Account> {
    db.with_tx(|tx| {
        let account = create_conn(tx, platform, username)?;
        attach_existing_games_conn(tx, &account)?;
        get_conn(tx, account.id)?
            .ok_or_else(|| AppError::Internal("Не удалось прочитать созданный аккаунт".to_string()))
    })
}

/// Привязывает к аккаунту уже лежащие в базе партии этого игрока, у которых
/// нет владельца. Так повторное подключение восстанавливает связь без
/// повторной загрузки архивов.
pub fn attach_existing_games_conn(conn: &Connection, account: &Account) -> AppResult<u32> {
    let source = match account.platform {
        AccountPlatform::Chesscom => "chesscom",
        AccountPlatform::Lichess => "lichess",
    };
    let changed = conn.execute(
        "UPDATE games SET account_id = ?1
         WHERE account_id IS NULL AND source = ?2 AND user_color IN ('white', 'black')
           AND (white_username = ?3 COLLATE NOCASE OR black_username = ?3 COLLATE NOCASE)",
        params![account.id, source, account.username],
    )?;
    Ok(changed as u32)
}

pub fn create_conn(
    conn: &Connection,
    platform: AccountPlatform,
    username: &str,
) -> AppResult<Account> {
    if let Some(existing) = find_conn(conn, platform, username)? {
        return Err(AppError::Validation(format!(
            "Аккаунт {} уже подключён",
            existing.username
        )));
    }
    let created_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO accounts (platform, username, created_at) VALUES (?1, ?2, ?3)",
        params![platform.as_str(), username.trim(), created_at],
    )?;
    let id = conn.last_insert_rowid();
    get_conn(conn, id)?.ok_or_else(|| {
        AppError::Internal("Не удалось прочитать только что созданный аккаунт".to_string())
    })
}

fn find_conn(
    conn: &Connection,
    platform: AccountPlatform,
    username: &str,
) -> AppResult<Option<Account>> {
    conn.query_row(
        &format!("{SELECT_ACCOUNT} WHERE a.platform = ?1 AND a.username = ?2 COLLATE NOCASE"),
        params![platform.as_str(), username.trim()],
        map_account,
    )
    .optional()
    .map_err(Into::into)
}

pub fn get_conn(conn: &Connection, id: i64) -> AppResult<Option<Account>> {
    conn.query_row(
        &format!("{SELECT_ACCOUNT} WHERE a.id = ?1"),
        [id],
        map_account,
    )
    .optional()
    .map_err(Into::into)
}

/// Удаляет аккаунт, но оставляет его партии в библиотеке: они становятся
/// «ручными» и доступны в режиме «Все аккаунты». Кэш архивов Chess.com
/// очищается, чтобы повторное подключение заново сверило историю.
pub fn remove(db: &Db, id: i64) -> AppResult<()> {
    db.with_tx(|tx| {
        let (platform, username): (String, String) = tx
            .query_row(
                "SELECT platform, username FROM accounts WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("Аккаунт #{id} не найден")))?;
        tx.execute("DELETE FROM accounts WHERE id = ?1", [id])?;
        if platform == AccountPlatform::Chesscom.as_str() {
            tx.execute(
                "DELETE FROM synced_archives WHERE username = ?1 COLLATE NOCASE",
                params![username],
            )?;
        }
        Ok(())
    })
}

pub fn touch_sync(db: &Db, id: i64, synced_at: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE accounts SET last_synced_at = ?1 WHERE id = ?2",
            params![synced_at, id],
        )?;
        Ok(())
    })
}

/// Возвращает аккаунт для ручного импорта: явно выбранный или единственный
/// подключённый. При нескольких аккаунтах владельца PGN определить нельзя,
/// поэтому партия остаётся без привязки.
pub fn resolve_import_account(db: &Db, requested: Option<i64>) -> AppResult<Option<Account>> {
    if let Some(id) = requested {
        return get(db, id).map(Some);
    }
    let accounts = list(db)?;
    if accounts.len() == 1 {
        return Ok(accounts.into_iter().next());
    }
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn creates_lists_and_removes_accounts() {
        let db = Db::in_memory().unwrap();
        let chess = create(&db, AccountPlatform::Chesscom, "Alice").unwrap();
        let lichess = create(&db, AccountPlatform::Lichess, "bob").unwrap();
        assert_eq!(list(&db).unwrap().len(), 2);
        assert_eq!(get(&db, chess.id).unwrap().username, "Alice");
        assert!(find_by_username(&db, AccountPlatform::Chesscom, "alice")
            .unwrap()
            .is_some());
        remove(&db, lichess.id).unwrap();
        assert_eq!(count(&db).unwrap(), 1);
    }

    #[test]
    fn rejects_duplicate_username_case_insensitively() {
        let db = Db::in_memory().unwrap();
        create(&db, AccountPlatform::Chesscom, "Alice").unwrap();
        let error = create(&db, AccountPlatform::Chesscom, "alice").unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
        create(&db, AccountPlatform::Lichess, "alice").unwrap();
    }

    #[test]
    fn removing_account_keeps_games_in_the_library() {
        let db = Db::in_memory().unwrap();
        let account = create(&db, AccountPlatform::Chesscom, "alice").unwrap();
        db.with(|conn| {
            conn.execute(
                "INSERT INTO games (external_id, source, pgn, white_username, black_username,
                    user_color, result, played_at, total_plies, account_id)
                 VALUES ('g1', 'chesscom', '1. e4 *', 'alice', 'bob', 'white', '*', 1, 1, ?1)",
                params![account.id],
            )?;
            Ok(())
        })
        .unwrap();
        remove(&db, account.id).unwrap();
        db.with(|conn| {
            let (games, owner): (i64, Option<i64>) = conn.query_row(
                "SELECT COUNT(*), MAX(account_id) FROM games WHERE external_id = 'g1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            assert_eq!(games, 1);
            assert_eq!(owner, None);
            Ok(())
        })
        .unwrap();
    }
}
