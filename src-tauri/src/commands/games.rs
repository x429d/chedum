use tauri::State;

use crate::db::{analyses_repo, games_repo, settings_repo};
use crate::domain::error::AppResult;
use crate::domain::game::{GameDetail, GameFilter, ImportSummary, PagedGames};
use crate::domain::stats::{AppSettings, OpeningEntry};
use crate::eco::EcoBook;
use crate::service::import;
use crate::state::AppState;

#[tauri::command]
pub async fn list_games(state: State<'_, AppState>, filter: GameFilter) -> AppResult<PagedGames> {
    games_repo::list_games(&state.db, &filter)
}

#[tauri::command]
pub async fn get_game(state: State<'_, AppState>, game_id: i64) -> AppResult<GameDetail> {
    let game = games_repo::get_game(&state.db, game_id)?;
    let analysis = analyses_repo::get_analysis(&state.db, game_id)?;
    let moves = analyses_repo::get_moves(&state.db, game_id)?;
    Ok(GameDetail {
        game,
        analysis,
        moves,
    })
}

#[tauri::command]
pub async fn delete_game(state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    games_repo::delete_game(&state.db, game_id)
}

#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppState>,
    game_id: i64,
    favorite: bool,
) -> AppResult<()> {
    games_repo::set_favorite(&state.db, game_id, favorite)
}

#[tauri::command]
pub async fn update_notes(
    state: State<'_, AppState>,
    game_id: i64,
    notes: String,
) -> AppResult<()> {
    games_repo::update_notes(&state.db, game_id, &notes)
}

#[tauri::command]
pub async fn save_user_comment(
    state: State<'_, AppState>,
    game_id: i64,
    ply: i32,
    comment: String,
) -> AppResult<()> {
    analyses_repo::save_move_comment(&state.db, game_id, ply, &comment)
}

#[tauri::command]
pub async fn import_pgn_text(state: State<'_, AppState>, text: String) -> AppResult<ImportSummary> {
    let settings = settings_repo::get_settings(&state.db)?;
    import::import_pgn_text(&state.db, &text, &settings.player_username)
}

#[tauri::command]
pub async fn import_pgn_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> AppResult<ImportSummary> {
    let settings = settings_repo::get_settings(&state.db)?;
    import::import_pgn_files(&state.db, &paths, &settings.player_username)
}

#[tauri::command]
pub async fn export_games(state: State<'_, AppState>, ids: Vec<i64>) -> AppResult<String> {
    import::export_pgn(&state.db, &ids)
}

#[tauri::command]
pub async fn export_games_to_file(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    path: String,
) -> AppResult<()> {
    import::export_pgn_to_path(&state.db, &ids, &path)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    settings_repo::get_settings(&state.db)
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> AppResult<()> {
    settings_repo::set_many(&state.db, &settings)
}

#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    settings_repo::set_setting(&state.db, &key, &value)
}

#[tauri::command]
pub async fn search_openings(query: String, limit: Option<u32>) -> AppResult<Vec<OpeningEntry>> {
    Ok(EcoBook::global().search(&query, limit.unwrap_or(20).clamp(1, 100) as usize))
}
