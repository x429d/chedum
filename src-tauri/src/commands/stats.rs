use tauri::State;

use crate::db::stats_repo;
use crate::domain::error::AppResult;
use crate::domain::stats::{AppInfo, HubStats};
use crate::eco::EcoBook;
use crate::state::AppState;

#[tauri::command]
pub async fn get_hub_stats(
    state: State<'_, AppState>,
    account_id: Option<i64>,
) -> AppResult<HubStats> {
    state
        .db
        .run(move |db| stats_repo::hub_stats(db, account_id))
        .await
}

#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    Ok(AppInfo {
        name: "Chenal".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        eco_openings: EcoBook::global().count(),
        max_analysis_workers: u32::try_from(crate::engine::default_workers()).unwrap_or(16),
        db_path: state.db_path.display().to_string(),
        log_path: state.log_path.clone(),
    })
}

/// Plain-text diagnostics suitable for a bug report. It intentionally contains
/// paths and engine status only, never game data.
#[tauri::command]
pub async fn get_diagnostics(state: State<'_, AppState>) -> AppResult<String> {
    let engine = state.engine.status();
    Ok(format!(
        "Chenal {}\nOS: {} {}\nDatabase: {}\nLog: {}\nEngine available: {}\nEngine: {}\nEngine path: {}\nEngine error: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        state.db_path.display(),
        state.log_path,
        engine.available,
        engine.name.unwrap_or_else(|| "unknown".to_string()),
        engine.path.unwrap_or_else(|| "not found".to_string()),
        engine.error.unwrap_or_else(|| "none".to_string()),
    ))
}
