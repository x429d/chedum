#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod analytics;
pub mod chesscom;
pub mod commands;
pub mod db;
pub mod domain;
pub mod eco;
pub mod engine;
pub mod lichess;
pub mod menu;
pub mod pgn;
pub mod service;
pub mod state;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;

use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;

use crate::db::{analyses_repo, seed, Db};
use crate::engine::EngineState;
use crate::state::AppState;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn init_logging(app: &tauri::AppHandle) -> Option<PathBuf> {
    let log_dir = app.path().app_log_dir().ok()?;
    std::fs::create_dir_all(&log_dir).ok()?;
    let appender = tracing_appender::rolling::daily(&log_dir, "chenal.log");
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let _ = LOG_GUARD.set(guard);
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_ansi(false)
                .with_target(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .try_init();
    Some(log_dir)
}

fn log_backfill(name: &str, result: crate::domain::error::AppResult<usize>) {
    match result {
        Ok(0) => {}
        Ok(count) => tracing::info!("{name}: {count} records updated"),
        Err(err) => tracing::warn!("{name} failed: {err}"),
    }
}

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let log_dir = init_logging(app.handle());
            // The menu intentionally has no Undo/Redo: otherwise macOS would
            // intercept Cmd+Z and Cmd+Shift+Z before the board gets them.
            app.handle().set_menu(menu::build(app.handle())?)?;
            let data_dir = app.path().app_data_dir().map_err(|err| {
                std::io::Error::other(format!("Could not resolve the data directory: {err}"))
            })?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("chenal.db");
            let db = Db::open(&db_path)?;
            let seeded = match seed::ensure_demo(&db) {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Loaded {count} demo games");
                    }
                    count
                }
                Err(err) => {
                    tracing::warn!("Could not load demo games: {err}");
                    0
                }
            };
            log_backfill(
                "Lichess accuracy backfill",
                analyses_repo::backfill_lichess_accuracy(&db, seeded > 0),
            );
            log_backfill(
                "Analysis config-key backfill",
                analyses_repo::backfill_config_keys(&db),
            );
            log_backfill(
                "FEN accuracy backfill",
                analyses_repo::backfill_fen_accuracy(&db),
            );
            log_backfill(
                "Analysis summary backfill",
                analyses_repo::backfill_summaries(&db),
            );
            log_backfill(
                "Key moments backfill",
                analyses_repo::backfill_key_moments(&db),
            );
            log_backfill(
                "Opening backfill",
                crate::service::openings::backfill_openings(&db).map(|count| count as usize),
            );
            let engine = EngineState::discover(app.handle());
            app.manage(AppState {
                db,
                engine,
                sync_cancel: AtomicBool::new(false),
                db_path,
                log_path: log_dir
                    .as_deref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "stderr".to_string()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::games::list_pending_analysis,
            commands::games::get_game,
            commands::games::delete_games,
            commands::games::toggle_favorite,
            commands::games::update_notes,
            commands::games::save_user_comment,
            commands::games::import_pgn_text,
            commands::games::import_pgn_files,
            commands::games::export_games_to_file,
            commands::games::get_settings,
            commands::games::save_settings,
            commands::games::search_openings,
            commands::analysis::analyze_game,
            commands::analysis::analyze_position,
            commands::analysis::stop_engine,
            commands::analysis::get_engine_status,
            commands::accounts::list_accounts,
            commands::accounts::add_account,
            commands::accounts::remove_account,
            commands::accounts::sync_account,
            commands::accounts::cancel_sync,
            commands::accounts::reset_sync_history,
            commands::stats::get_hub_stats,
            commands::stats::get_app_info,
            commands::stats::get_diagnostics,
        ]);
    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(err) => {
            tracing::error!("Could not create the application: {err}");
            return;
        }
    };
    app.run(|handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = handle.state::<AppState>();
            tauri::async_runtime::block_on(async {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    state.engine.shutdown(),
                )
                .await;
            });
        }
    });
}
