use std::sync::atomic::Ordering;

use chrono::Utc;
use tauri::ipc::Channel;
use tauri::State;

use crate::chesscom::client::ChessComClient;
use crate::db::{accounts_repo, settings_repo};
use crate::domain::account::{normalize_username, AccountPlatform};
use crate::domain::error::{AppError, AppResult};
use crate::domain::{Account, SyncProgress, SyncResult};
use crate::lichess::client::LichessClient;
use crate::state::AppState;

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> AppResult<Vec<Account>> {
    state.db.run(accounts_repo::list).await
}

#[tauri::command]
pub async fn add_account(
    state: State<'_, AppState>,
    platform: String,
    username: String,
) -> AppResult<Account> {
    let platform = AccountPlatform::from_str_opt(platform.trim())
        .ok_or_else(|| AppError::Validation("Unknown account platform".to_string()))?;
    let username = normalize_username(platform, &username)?;
    state
        .db
        .run(move |db| accounts_repo::create(db, platform, &username))
        .await
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, account_id: i64) -> AppResult<()> {
    state
        .db
        .run(move |db| {
            accounts_repo::remove(db, account_id)?;
            let settings = settings_repo::get_settings(db)?;
            if settings.active_account_id == Some(account_id) {
                settings_repo::set_setting(db, "active_account_id", "")?;
            }
            Ok(())
        })
        .await
}

#[tauri::command]
pub async fn sync_account(
    state: State<'_, AppState>,
    account_id: i64,
    force: bool,
    on_progress: Channel<SyncProgress>,
) -> AppResult<SyncResult> {
    state.sync_cancel.store(false, Ordering::SeqCst);
    let account = state
        .db
        .run(move |db| accounts_repo::get(db, account_id))
        .await?;
    let result = match account.platform {
        AccountPlatform::Chesscom => {
            let client = ChessComClient::new()?;
            crate::chesscom::sync::sync_user(
                &state.db,
                &client,
                account.id,
                &account.username,
                force,
                &state.sync_cancel,
                |progress| {
                    let _ = on_progress.send(progress.clone());
                },
            )
            .await?
        }
        AccountPlatform::Lichess => {
            let client = LichessClient::new()?;
            crate::lichess::sync::sync_user(
                &state.db,
                &client,
                account.id,
                &account.username,
                account.last_synced_at.as_deref(),
                force,
                &state.sync_cancel,
                |progress| {
                    let _ = on_progress.send(progress.clone());
                },
            )
            .await?
        }
    };
    accounts_repo::touch_sync(&state.db, account.id, &Utc::now().to_rfc3339())?;
    Ok(result)
}

#[tauri::command]
pub async fn cancel_sync(state: State<'_, AppState>) -> AppResult<()> {
    state.sync_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn reset_sync_history(state: State<'_, AppState>, account_id: i64) -> AppResult<()> {
    state
        .db
        .run(move |db| accounts_repo::reset_sync_state(db, account_id))
        .await
}
