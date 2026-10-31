use tauri::State;

use crate::db::{analyses_repo, games_delete, games_repo, settings_repo};
use crate::domain::error::AppResult;
use crate::domain::game::{GameDetail, GameFilter, ImportSummary, PagedGames, PendingAnalysis};
use crate::domain::stats::{AppSettings, OpeningEntry};
use crate::eco::EcoBook;
use crate::service::import;
use crate::state::AppState;

#[tauri::command]
pub async fn list_games(state: State<'_, AppState>, filter: GameFilter) -> AppResult<PagedGames> {
    state
        .db
        .run(move |db| games_repo::list_games(db, &filter))
        .await
}

#[tauri::command]
pub async fn list_pending_analysis(
    state: State<'_, AppState>,
    filter: GameFilter,
) -> AppResult<PendingAnalysis> {
    state
        .db
        .run(move |db| {
            let settings = settings_repo::get_settings(db)?;
            let config_key = crate::service::queue::target_config_key(db, &settings)?;
            games_repo::pending_analysis(
                db,
                &filter,
                &config_key,
                settings.analysis_depth,
                settings.multipv,
            )
        })
        .await
}

#[tauri::command]
pub async fn get_game(state: State<'_, AppState>, game_id: i64) -> AppResult<GameDetail> {
    state
        .db
        .run(move |db| {
            let game = games_repo::get_game(db, game_id)?;
            let analysis = analyses_repo::get_analysis(db, game_id)?;
            let moves = analyses_repo::get_moves(db, game_id)?;
            Ok(GameDetail {
                game,
                analysis,
                moves,
            })
        })
        .await
}

#[tauri::command]
pub async fn delete_games(state: State<'_, AppState>, ids: Vec<i64>) -> AppResult<u32> {
    state
        .db
        .run(move |db| games_delete::delete_games(db, &ids))
        .await
}

#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppState>,
    game_id: i64,
    favorite: bool,
) -> AppResult<()> {
    state
        .db
        .run(move |db| games_repo::set_favorite(db, game_id, favorite))
        .await
}

#[tauri::command]
pub async fn update_notes(
    state: State<'_, AppState>,
    game_id: i64,
    notes: String,
) -> AppResult<()> {
    state
        .db
        .run(move |db| games_repo::update_notes(db, game_id, &notes))
        .await
}

#[tauri::command]
pub async fn save_user_comment(
    state: State<'_, AppState>,
    game_id: i64,
    ply: i32,
    comment: String,
) -> AppResult<()> {
    state
        .db
        .run(move |db| analyses_repo::save_move_comment(db, game_id, ply, &comment))
        .await
}

#[tauri::command]
pub async fn import_pgn_text(
    state: State<'_, AppState>,
    text: String,
    account_id: Option<i64>,
) -> AppResult<ImportSummary> {
    state
        .db
        .run(move |db| import::import_pgn_text(db, &text, account_id))
        .await
}

#[tauri::command]
pub async fn import_pgn_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
    account_id: Option<i64>,
) -> AppResult<ImportSummary> {
    state
        .db
        .run(move |db| import::import_pgn_files(db, &paths, account_id))
        .await
}

#[tauri::command]
pub async fn export_games_to_file(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    path: String,
) -> AppResult<()> {
    state
        .db
        .run(move |db| import::export_pgn_to_path(db, &ids, &path))
        .await
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    state.db.run(settings_repo::get_settings).await
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> AppResult<()> {
    state
        .db
        .run(move |db| settings_repo::set_many(db, &settings))
        .await
}

#[tauri::command]
pub async fn search_openings(query: String, limit: Option<u32>) -> AppResult<Vec<OpeningEntry>> {
    Ok(EcoBook::global().search(&query, limit.unwrap_or(20).clamp(1, 100) as usize))
}
