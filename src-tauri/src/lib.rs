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
pub mod pgn;
pub mod service;
pub mod state;

use tauri::Manager;

use crate::db::{analyses_repo, seed, Db};
use crate::engine::EngineState;
use crate::state::AppState;

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().map_err(|err| {
                std::io::Error::other(format!("Не удалось определить каталог данных: {err}"))
            })?;
            std::fs::create_dir_all(&data_dir)?;
            let db = Db::open(&data_dir.join("chenal.db"))?;
            let seeded = match seed::ensure_demo(&db) {
                Ok(count) => {
                    if count > 0 {
                        eprintln!("Chenal: загружено демонстрационных партий — {count}");
                    }
                    count
                }
                Err(err) => {
                    eprintln!("Chenal: не удалось загрузить демо-данные: {err}");
                    0
                }
            };
            match analyses_repo::backfill_lichess_accuracy(&db, seeded > 0) {
                Ok(count) if count > 0 => {
                    eprintln!("Chenal: точность пересчитана по методу Lichess — {count}");
                }
                Ok(_) => {}
                Err(err) => eprintln!("Chenal: не удалось пересчитать точность: {err}"),
            }
            match analyses_repo::backfill_summaries(&db) {
                Ok(count) if count > 0 => {
                    eprintln!("Chenal: резюме партий пересобраны из метрик — {count}");
                }
                Ok(_) => {}
                Err(err) => eprintln!("Chenal: не удалось пересобрать резюме: {err}"),
            }
            match crate::service::openings::backfill_openings(&db) {
                Ok(count) if count > 0 => {
                    eprintln!("Chenal: дебюты пересчитаны по позициям — {count}");
                }
                Ok(_) => {}
                Err(err) => eprintln!("Chenal: не удалось пересчитать дебюты: {err}"),
            }
            let engine = EngineState::discover(app.handle());
            app.manage(AppState { db, engine });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::games::list_games,
            commands::games::list_pending_analysis,
            commands::games::get_game,
            commands::games::delete_game,
            commands::games::delete_games,
            commands::games::toggle_favorite,
            commands::games::update_notes,
            commands::games::save_user_comment,
            commands::games::import_pgn_text,
            commands::games::import_pgn_files,
            commands::games::export_games,
            commands::games::export_games_to_file,
            commands::games::get_settings,
            commands::games::save_settings,
            commands::games::set_setting,
            commands::games::search_openings,
            commands::analysis::analyze_game,
            commands::analysis::analyze_position,
            commands::analysis::stop_engine,
            commands::analysis::get_engine_status,
            commands::sync::sync_chesscom,
            commands::sync::get_sync_status,
            commands::stats::get_hub_stats,
            commands::stats::get_app_info,
        ]);
    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("Chenal: не удалось создать приложение: {err}");
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
