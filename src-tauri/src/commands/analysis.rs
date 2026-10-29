use tauri::ipc::Channel;
use tauri::State;

use crate::db::settings_repo;
use crate::domain::analysis::AnalysisProgress;
use crate::domain::error::AppResult;
use crate::domain::game::GameDetail;
use crate::domain::stats::EngineStatus;
use crate::engine::EngineUpdate;
use crate::service::analysis;
use crate::state::AppState;

#[tauri::command]
pub async fn analyze_game(
    state: State<'_, AppState>,
    game_id: i64,
    depth: Option<u32>,
    multipv: Option<u32>,
    on_progress: Channel<AnalysisProgress>,
) -> AppResult<GameDetail> {
    let settings = settings_repo::get_settings(&state.db)?;
    let depth = depth.unwrap_or(settings.analysis_depth).clamp(6, 24);
    let multipv = multipv.unwrap_or(settings.multipv).clamp(1, 3);
    let lease = state.engine.acquire(settings.analysis_threads).await?;
    analysis::analyze_game(
        &state.db,
        lease.manager(),
        game_id,
        depth,
        multipv,
        Some(&on_progress),
    )
    .await
}

#[tauri::command]
pub async fn analyze_position(
    state: State<'_, AppState>,
    fen: String,
    depth: Option<u32>,
    multipv: Option<u32>,
    on_progress: Channel<EngineUpdate>,
) -> AppResult<EngineUpdate> {
    let settings = settings_repo::get_settings(&state.db)?;
    let lease = state.engine.acquire(settings.analysis_threads).await?;
    let engine = lease.manager();
    let depth = depth.unwrap_or(16).clamp(6, 30);
    let multipv = multipv.unwrap_or(3).clamp(1, 3);
    let _job = engine.begin_job().await?;
    engine
        .analyze(&fen, depth, multipv, |update| {
            let _ = on_progress.send(update.clone());
        })
        .await
}

#[tauri::command]
pub async fn stop_engine(state: State<'_, AppState>) -> AppResult<()> {
    state.engine.request_stop().await;
    Ok(())
}

#[tauri::command]
pub async fn get_engine_status(state: State<'_, AppState>) -> AppResult<EngineStatus> {
    state.engine.probe().await;
    Ok(state.engine.status())
}
