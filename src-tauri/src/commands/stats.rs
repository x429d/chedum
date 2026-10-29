use tauri::State;

use crate::db::stats_repo;
use crate::domain::error::AppResult;
use crate::domain::stats::{AppInfo, HubStats};
use crate::eco::EcoBook;
use crate::state::AppState;

#[tauri::command]
pub async fn get_hub_stats(state: State<'_, AppState>) -> AppResult<HubStats> {
    stats_repo::hub_stats(&state.db)
}

#[tauri::command]
pub async fn get_app_info() -> AppResult<AppInfo> {
    Ok(AppInfo {
        name: "Chenal".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        eco_openings: EcoBook::global().count(),
        max_analysis_workers: u32::try_from(crate::engine::default_workers()).unwrap_or(16),
    })
}
