use std::path::PathBuf;
use std::sync::Mutex;

use tauri::Manager;
use tokio::sync::OnceCell;

use crate::domain::error::{AppError, AppResult};
use crate::domain::stats::EngineStatus;
use crate::engine::manager::EngineManager;

const BINARY_NAME: &str = "stockfish-aarch64-apple-darwin";

pub struct EngineState {
    path: Option<PathBuf>,
    cell: OnceCell<EngineManager>,
    init_error: Mutex<Option<String>>,
}

impl EngineState {
    pub fn discover(app: &tauri::AppHandle) -> Self {
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(resource_dir) = app.path().resource_dir() {
            candidates.push(resource_dir.join("binaries").join(BINARY_NAME));
        }
        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(BINARY_NAME),
        );
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("binaries").join(BINARY_NAME));
            candidates.push(cwd.join("src-tauri").join("binaries").join(BINARY_NAME));
        }
        candidates.push(PathBuf::from("/opt/homebrew/bin/stockfish"));
        candidates.push(PathBuf::from("/usr/local/bin/stockfish"));
        let path = candidates.into_iter().find(|candidate| candidate.is_file());
        EngineState {
            path,
            cell: OnceCell::new(),
            init_error: Mutex::new(None),
        }
    }

    pub async fn manager(&self) -> AppResult<&EngineManager> {
        let path = self.path.as_deref().ok_or_else(|| {
            AppError::Engine(
                "Stockfish не найден. Установите его через brew install stockfish.".to_string(),
            )
        })?;
        let result = self
            .cell
            .get_or_try_init(|| EngineManager::spawn(path))
            .await;
        match result {
            Ok(manager) => Ok(manager),
            Err(err) => {
                self.record_error(&err);
                Err(err)
            }
        }
    }

    pub fn status(&self) -> EngineStatus {
        EngineStatus {
            available: self.path.is_some(),
            path: self.path.as_ref().map(|path| path.display().to_string()),
            name: self.cell.get().map(|manager| manager.name().to_string()),
            error: self.init_error.lock().ok().and_then(|guard| guard.clone()),
        }
    }

    pub fn request_stop(&self) {
        if let Some(manager) = self.cell.get() {
            manager.request_stop();
        }
    }

    pub async fn shutdown(&self) {
        self.request_stop();
        if let Some(manager) = self.cell.get() {
            manager.shutdown().await;
        }
    }

    fn record_error(&self, err: &AppError) {
        if let Ok(mut guard) = self.init_error.lock() {
            *guard = Some(err.to_string());
        }
    }
}
