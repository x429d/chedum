use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::db::connection::Db;
use crate::engine::EngineState;

pub struct AppState {
    pub db: Db,
    pub engine: EngineState,
    /// Shared cancellation flag for long account synchronizations.
    pub sync_cancel: AtomicBool,
    pub db_path: PathBuf,
    pub log_path: String,
}
