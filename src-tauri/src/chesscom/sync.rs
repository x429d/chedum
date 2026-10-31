use chrono::{Datelike, Utc};

use crate::chesscom::client::{ChessComApi, RemoteGame};
use crate::db::archives_repo::{self, ArchiveRecord};
use crate::db::connection::Db;
use crate::db::games_repo::insert_game;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{GameSource, TimeClass};
use crate::domain::{SyncProgress, SyncResult};
use crate::pgn::{build_draft, parse_single};
use crate::service::hashing::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveRef {
    pub url: String,
    pub year: i32,
    pub month: u32,
}

/// Год и месяц из URL архива Chess.com (`.../games/2024/05`).
pub fn parse_archive_url(url: &str) -> Option<(i32, u32)> {
    let mut parts = url.trim_end_matches('/').rsplit('/');
    let month = parts.next()?.parse::<u32>().ok()?;
    let year = parts.next()?.parse::<i32>().ok()?;
    if (1..=12).contains(&month) && (2000..=2100).contains(&year) {
        Some((year, month))
    } else {
        None
    }
}

/// Инкрементальный план: пропускает полностью синхронизированные прошлые архивы.
pub fn plan_archives(
    existing: &[ArchiveRecord],
    archives: &[String],
    now_year: i32,
    now_month: u32,
    force: bool,
) -> Vec<ArchiveRef> {
    let mut plan = Vec::new();
    for url in archives {
        let Some((year, month)) = parse_archive_url(url) else {
            continue;
        };
        let record = existing.iter().find(|record| record.archive_url == *url);
        let is_current = year == now_year && month == now_month;
        let needed = force
            || record.is_none()
            || record.is_some_and(|record| !record.is_fully_synced)
            || is_current;
        if needed {
            plan.push(ArchiveRef {
                url: url.clone(),
                year,
                month,
            });
        }
    }
    plan.sort_by_key(|archive| (archive.year, archive.month));
    plan
}

pub async fn sync_user<A, F>(
    db: &Db,
    api: &A,
    account_id: i64,
    username: &str,
    force: bool,
    mut on_progress: F,
) -> AppResult<SyncResult>
where
    A: ChessComApi + ?Sized,
    F: FnMut(&SyncProgress),
{
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::Validation("Укажите ник Chess.com".to_string()));
    }
    let now = Utc::now();
    let mut result = SyncResult {
        account_id,
        platform: crate::domain::AccountPlatform::Chesscom,
        username: username.to_string(),
        archives_processed: 0,
        games_imported: 0,
        games_skipped: 0,
    };
    emit(
        &mut on_progress,
        "archives",
        "Получение списка архивов Chess.com",
        0,
        0,
        0,
        0.0,
    );
    let archives = api.archives(username).await?;
    let existing = archives_repo::list_for_user(db, username)?;
    let plan = plan_archives(&existing, &archives, now.year(), now.month(), force);
    let total = plan.len() as u32;
    if total == 0 {
        emit(
            &mut on_progress,
            "done",
            "Новых архивов нет",
            0,
            0,
            0,
            100.0,
        );
        return Ok(result);
    }
    let mut done = 0u32;
    for archive in plan {
        done += 1;
        emit(
            &mut on_progress,
            "archive",
            &format!("Загрузка архива {:04}-{:02}", archive.year, archive.month),
            done,
            total,
            result.games_imported,
            (done as f64 - 1.0) / f64::from(total) * 90.0,
        );
        let remote_games = api.archive_games(&archive.url).await?;
        let mut drafts = Vec::new();
        for remote in remote_games {
            match remote_to_draft(&remote, username, account_id) {
                Some(draft) => drafts.push(draft),
                None => result.games_skipped += 1,
            }
        }
        let total_games = drafts.len() as u32;
        let inserted = db.with_tx(|tx| {
            let mut count = 0u32;
            for draft in &drafts {
                if insert_game(tx, draft)?.is_some() {
                    count += 1;
                }
            }
            let fully_synced = archive.year < now.year()
                || (archive.year == now.year() && archive.month < now.month());
            archives_repo::upsert_conn(
                tx,
                username,
                &archive.url,
                archive.year,
                archive.month as i32,
                fully_synced,
                &Utc::now().to_rfc3339(),
            )?;
            Ok(count)
        })?;
        result.games_imported += inserted;
        result.games_skipped += total_games - inserted;
        result.archives_processed += 1;
        emit(
            &mut on_progress,
            "archive",
            &format!(
                "Архив {:04}-{:02}: +{} партий",
                archive.year, archive.month, inserted
            ),
            done,
            total,
            result.games_imported,
            done as f64 / f64::from(total) * 90.0,
        );
    }
    emit(
        &mut on_progress,
        "done",
        &format!("Импортировано {} партий", result.games_imported),
        total,
        total,
        result.games_imported,
        100.0,
    );
    Ok(result)
}

fn remote_to_draft(
    remote: &RemoteGame,
    username: &str,
    account_id: i64,
) -> Option<crate::domain::game::GameDraft> {
    if !remote.rules.is_empty() && remote.rules != "chess" {
        return None;
    }
    let parsed = parse_single(&remote.pgn).ok()?;
    let external_id = if remote.url.trim().is_empty() {
        format!("chesscom-{}", sha256_hex(&remote.pgn))
    } else {
        remote.url.trim().to_string()
    };
    let mut draft = build_draft(
        &parsed,
        GameSource::Chesscom,
        external_id,
        username,
        TimeClass::from_str_opt(&remote.time_class),
        Some(&remote.pgn),
    );
    draft.account_id = Some(account_id);
    if remote.end_time > 0 {
        draft.played_at = remote.end_time;
    }
    Some(draft)
}

#[allow(clippy::too_many_arguments)]
fn emit<F>(
    on_progress: &mut F,
    phase: &str,
    message: &str,
    done: u32,
    total: u32,
    imported: u32,
    percent: f64,
) where
    F: FnMut(&SyncProgress),
{
    on_progress(&SyncProgress {
        phase: phase.to_string(),
        message: message.to_string(),
        archives_done: done,
        archives_total: total,
        games_imported: imported,
        percent,
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "sync_tests.rs"]
mod tests;
