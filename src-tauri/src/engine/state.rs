use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::Manager;
use tokio::sync::Mutex as AsyncMutex;

use crate::domain::error::{AppError, AppResult};
use crate::domain::stats::EngineStatus;
use crate::engine::manager::EngineManager;
use crate::engine::pool::{hash_mb_for, workers_from_setting, EnginePool, PooledEngine};

const BINARY_NAME: &str = "stockfish-aarch64-apple-darwin";

pub struct EngineState {
    path: Option<PathBuf>,
    pool: AsyncMutex<Option<(usize, Arc<EnginePool>)>>,
    name: Mutex<Option<String>>,
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
            pool: AsyncMutex::new(None),
            name: Mutex::new(None),
            init_error: Mutex::new(None),
        }
    }

    /// Пул под текущую настройку. При смене размера создаётся новый пул;
    /// старый доживает уже запущенные партии и освобождает процессы сам.
    pub async fn pool_for(&self, setting: u32) -> AppResult<Arc<EnginePool>> {
        let workers = workers_from_setting(setting);
        let mut guard = self.pool.lock().await;
        if let Some((size, pool)) = guard.as_ref() {
            if *size == workers {
                return Ok(Arc::clone(pool));
            }
        }
        let path = self.path.as_deref().ok_or_else(|| {
            AppError::Engine(
                "Stockfish не найден. Установите его через brew install stockfish.".to_string(),
            )
        })?;
        let pool = match EnginePool::spawn(path, workers, hash_mb_for(workers)).await {
            Ok(pool) => pool,
            Err(err) => {
                self.record_error(&err);
                return Err(err);
            }
        };
        if let Ok(mut name) = self.name.lock() {
            *name = pool.first_name();
        }
        *guard = Some((workers, Arc::clone(&pool)));
        Ok(pool)
    }

    /// Арендует один процесс движка на время анализа партии или позиции.
    pub async fn acquire(&self, setting: u32) -> AppResult<PooledEngine> {
        let pool = self.pool_for(setting).await?;
        Ok(pool.acquire().await)
    }

    /// Короткая проверка движка для панели диагностики: один процесс с
    /// малым хешем, без создания всего пула.
    pub async fn probe(&self) {
        let known = self.name.lock().ok().and_then(|guard| guard.clone());
        if known.is_some() {
            return;
        }
        let Some(path) = self.path.as_deref() else {
            return;
        };
        match EngineManager::spawn_with(path, 1, 16).await {
            Ok(manager) => {
                if let Ok(mut name) = self.name.lock() {
                    *name = Some(manager.name().to_string());
                }
                manager.shutdown().await;
            }
            Err(err) => self.record_error(&err),
        }
    }

    pub fn status(&self) -> EngineStatus {
        EngineStatus {
            available: self.path.is_some(),
            path: self.path.as_ref().map(|path| path.display().to_string()),
            name: self.name.lock().ok().and_then(|guard| guard.clone()),
            error: self.init_error.lock().ok().and_then(|guard| guard.clone()),
        }
    }

    pub async fn request_stop(&self) {
        if let Some((_, pool)) = self.pool.lock().await.as_ref() {
            pool.request_stop();
        }
    }

    pub async fn shutdown(&self) {
        let pool = self.pool.lock().await.take();
        if let Some((_, pool)) = pool {
            pool.request_stop();
            pool.shutdown().await;
        }
    }

    fn record_error(&self, err: &AppError) {
        if let Ok(mut guard) = self.init_error.lock() {
            *guard = Some(err.to_string());
        }
    }
}
