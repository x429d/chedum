use tauri::ipc::Channel;
use tauri::State;

use crate::chesscom::client::ChessComClient;
use crate::chesscom::sync::sync_user;
use crate::db::{archives_repo, settings_repo};
use crate::domain::error::AppResult;
use crate::domain::{SyncProgress, SyncResult};
use crate::state::AppState;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveView {
    pub username: String,
    pub archive_url: String,
    pub year: i32,
    pub month: i32,
    pub is_fully_synced: bool,
    pub last_synced_at: String,
}

#[tauri::command]
pub async fn sync_chesscom(
    state: State<'_, AppState>,
    username: String,
    force: bool,
    on_progress: Channel<SyncProgress>,
) -> AppResult<SyncResult> {
    let settings = settings_repo::get_settings(&state.db)?;
    let username = if username.trim().is_empty() {
        settings.player_username.clone()
    } else {
        username
    };
    let client = ChessComClient::new()?;
    let result = sync_user(&state.db, &client, &username, force, |progress| {
        let _ = on_progress.send(progress.clone());
    })
    .await?;
    settings_repo::set_setting(&state.db, "player_username", &result.username)?;
    Ok(result)
}

#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, AppState>,
    username: String,
) -> AppResult<Vec<ArchiveView>> {
    let settings = settings_repo::get_settings(&state.db)?;
    let username = if username.trim().is_empty() {
        settings.player_username
    } else {
        username
    };
    let archives = archives_repo::list_for_user(&state.db, username.trim())?;
    Ok(archives
        .into_iter()
        .map(|archive| ArchiveView {
            username: archive.username,
            archive_url: archive.archive_url,
            year: archive.year,
            month: archive.month,
            is_fully_synced: archive.is_fully_synced,
            last_synced_at: archive.last_synced_at,
        })
        .collect())
}
