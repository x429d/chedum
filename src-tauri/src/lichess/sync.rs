use std::sync::atomic::{AtomicBool, Ordering};

use chrono::DateTime;

use crate::db::connection::Db;
use crate::db::games_repo::insert_game;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{GameDraft, GameSource};
use crate::domain::{AccountPlatform, SyncProgress, SyncResult};
use crate::lichess::client::LichessApi;
use crate::pgn::{build_draft, is_bot_game, parse_pgn, ParsedGame};
use crate::service::hashing::sha256_hex;

/// Small overlap so a game that finished at the same second as the previous
/// sync is not lost.
const OVERLAP_MS: i64 = 5 * 60 * 1000;
const MAX_ERROR_SAMPLES: usize = 20;

pub fn since_ms(last_synced_at: Option<&str>) -> Option<i64> {
    let raw = last_synced_at?;
    let parsed = DateTime::parse_from_rfc3339(raw).ok()?;
    Some(parsed.timestamp_millis() - OVERLAP_MS)
}

fn check_cancelled(cancel: &AtomicBool) -> AppResult<()> {
    if cancel.load(Ordering::SeqCst) {
        return Err(AppError::Cancelled(
            "Lichess synchronization cancelled".to_string(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn sync_user<A, F>(
    db: &Db,
    api: &A,
    account_id: i64,
    username: &str,
    last_synced_at: Option<&str>,
    force: bool,
    cancel: &AtomicBool,
    mut on_progress: F,
) -> AppResult<SyncResult>
where
    A: LichessApi + ?Sized,
    F: FnMut(&SyncProgress),
{
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::Validation("Enter a Lichess username".to_string()));
    }
    check_cancelled(cancel)?;
    let mut result = SyncResult {
        account_id,
        platform: AccountPlatform::Lichess,
        username: username.to_string(),
        archives_processed: 0,
        games_imported: 0,
        duplicates: 0,
        filtered: 0,
        errors: Vec::new(),
    };
    let since = if force {
        None
    } else {
        since_ms(last_synced_at)
    };
    emit(
        &mut on_progress,
        "download",
        if since.is_some() {
            "Downloading new Lichess games"
        } else {
            "Downloading all Lichess games (the first run may take a few minutes)"
        },
        0,
        0,
        0,
        5.0,
    );
    let pgn = api.user_games_pgn(username, since).await?;
    check_cancelled(cancel)?;
    if pgn.trim().is_empty() {
        emit(&mut on_progress, "done", "No new games", 0, 0, 0, 100.0);
        return Ok(result);
    }
    emit(
        &mut on_progress,
        "parse",
        "Parsing the downloaded Lichess PGN",
        0,
        0,
        0,
        35.0,
    );
    let report = parse_pgn(&pgn);
    for error in report.errors.iter().take(MAX_ERROR_SAMPLES) {
        result.errors.push(format!("Lichess PGN: {error}"));
    }
    let mut drafts: Vec<GameDraft> = Vec::new();
    for parsed in report.games {
        match draft_for_game(&parsed, username, account_id) {
            Ok(Some(draft)) => drafts.push(draft),
            Ok(None) => result.filtered += 1,
            Err(err) => {
                if result.errors.len() < MAX_ERROR_SAMPLES {
                    result.errors.push(err);
                }
            }
        }
    }
    let total = drafts.len() as u32;
    emit(
        &mut on_progress,
        "import",
        &format!("Importing Lichess games: {total}"),
        0,
        total,
        0,
        55.0,
    );
    check_cancelled(cancel)?;
    let inserted = db.with_tx(|tx| {
        let mut count = 0u32;
        for draft in &drafts {
            if insert_game(tx, draft)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    })?;
    result.games_imported = inserted;
    result.duplicates = total - inserted;
    emit(
        &mut on_progress,
        "done",
        &format!("Imported {} games", result.games_imported),
        total,
        total,
        result.games_imported,
        100.0,
    );
    Ok(result)
}

/// `Ok(None)` means an intentionally filtered game; `Err` means the game PGN
/// could not be parsed and should be reported.
fn draft_for_game(
    parsed: &ParsedGame,
    username: &str,
    account_id: i64,
) -> Result<Option<GameDraft>, String> {
    if is_bot_game(parsed) {
        return Ok(None);
    }
    let variant = parsed.tag("Variant").unwrap_or("Standard");
    if !matches!(variant, "Standard" | "From Position") || parsed.moves.is_empty() {
        return Ok(None);
    }
    let pgn = parsed.to_pgn();
    let external_id = parsed
        .tag("GameId")
        .filter(|id| !id.trim().is_empty())
        .map(|id| format!("https://lichess.org/{}", id.trim()))
        .unwrap_or_else(|| format!("lichess-{}", sha256_hex(&pgn)));
    let mut draft = build_draft(
        parsed,
        GameSource::Lichess,
        external_id,
        username,
        None,
        None,
    );
    draft.account_id = Some(account_id);
    Ok(Some(draft))
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
