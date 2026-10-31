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
    accounts_repo::list(&state.db)
}

#[tauri::command]
pub async fn add_account(
    state: State<'_, AppState>,
    platform: String,
    username: String,
) -> AppResult<Account> {
    let platform = AccountPlatform::from_str_opt(platform.trim())
        .ok_or_else(|| AppError::Validation("Неизвестная площадка аккаунта".to_string()))?;
    let username = normalize_username(platform, &username)?;
    accounts_repo::create(&state.db, platform, &username)
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, account_id: i64) -> AppResult<()> {
    accounts_repo::remove(&state.db, account_id)?;
    let settings = settings_repo::get_settings(&state.db)?;
    if settings.active_account_id == Some(account_id) {
        settings_repo::set_setting(&state.db, "active_account_id", "")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_account(
    state: State<'_, AppState>,
    account_id: i64,
    force: bool,
    on_progress: Channel<SyncProgress>,
) -> AppResult<SyncResult> {
    let account = accounts_repo::get(&state.db, account_id)?;
    let result = match account.platform {
        AccountPlatform::Chesscom => {
            let client = ChessComClient::new()?;
            crate::chesscom::sync::sync_user(
                &state.db,
                &client,
                account.id,
                &account.username,
                force,
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
