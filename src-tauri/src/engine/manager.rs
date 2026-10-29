use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, MutexGuard};
use tokio::time::timeout;

use crate::analytics::Score;
use crate::domain::error::{AppError, AppResult};
use crate::engine::process::EngineProcess;
use crate::engine::pv::{line_to_san, move_to_san};
use crate::engine::types::{EngineLine, EngineUpdate};
use crate::engine::uci::{parse_bestmove, parse_info};

const THROTTLE: Duration = Duration::from_millis(75);
const CANCELLED: &str = "Анализ отменён";
/// Сколько новая задача ждёт освобождения движка, прежде чем получить `busy`.
/// Короткая пауза покрывает остановку live-анализа, долгая очередь — ошибка.
const JOB_WAIT: Duration = Duration::from_secs(2);

/// Право на использование движка. Живёт весь анализ партии, поэтому
/// параллельные пакеты не могут незаметно встать в бесконечную очередь.
pub struct EngineJob<'a> {
    _slot: MutexGuard<'a, ()>,
}

pub struct EngineManager {
    path: PathBuf,
    process: Arc<Mutex<EngineProcess>>,
    cancel: Arc<AtomicBool>,
    slot: Mutex<()>,
    name: String,
    version: String,
}

impl EngineManager {
    pub async fn spawn(path: &Path) -> AppResult<Self> {
        EngineManager::spawn_with(path, 2, 128).await
    }

    /// Один процесс пула: обычно `threads = 1` и умеренный `Hash`.
    pub async fn spawn_with(path: &Path, threads: usize, hash_mb: u32) -> AppResult<Self> {
        let process = EngineProcess::spawn_with(path, threads, hash_mb).await?;
        let name = process.name().to_string();
        let version = version_from_name(&name);
        Ok(EngineManager {
            path: path.to_path_buf(),
            process: Arc::new(Mutex::new(process)),
            cancel: Arc::new(AtomicBool::new(false)),
            slot: Mutex::new(()),
            name,
            version,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Версия из `id name Stockfish 19` — «19». Хранится в записи анализа.
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Начинает новую задачу анализа: одна задача на движок.
    /// Сбрасывает флаг отмены только здесь, а не на каждой позиции.
    pub async fn begin_job(&self) -> AppResult<EngineJob<'_>> {
        let Ok(slot) = timeout(JOB_WAIT, self.slot.lock()).await else {
            return Err(AppError::Busy(
                "Движок уже анализирует другую партию".to_string(),
            ));
        };
        self.cancel.store(false, Ordering::SeqCst);
        Ok(EngineJob { _slot: slot })
    }

    /// Анализирует позицию, транслируя обновления с троттлингом 75 мс.
    /// Вызывается только внутри задачи, открытой `begin_job`.
    pub async fn analyze<F>(
        &self,
        fen: &str,
        depth: u32,
        multipv: u32,
        mut on_update: F,
    ) -> AppResult<EngineUpdate>
    where
        F: FnMut(&EngineUpdate),
    {
        let depth = depth.clamp(4, 30);
        let multipv = multipv.clamp(1, 3);
        if self.is_cancelled() {
            return Err(AppError::Engine(CANCELLED.to_string()));
        }
        let mut process = self.process.lock().await;
        if !process.is_running() {
            *process = EngineProcess::spawn(&self.path).await?;
        }
        let outcome = self
            .search(&mut process, fen, depth, multipv, &mut on_update)
            .await;
        match &outcome {
            Ok(_) => {}
            Err(err) if is_cancel_error(err) => {}
            Err(_) => {
                // Сдвинутый или зависший UCI-поток нельзя переиспользовать:
                // следующий вызов начал бы искать поверх прошлого search.
                match EngineProcess::spawn(&self.path).await {
                    Ok(fresh) => *process = fresh,
                    Err(restart) => {
                        eprintln!("Chenal: не удалось перезапустить Stockfish: {restart}");
                    }
                }
            }
        }
        outcome
    }

    async fn search<F>(
        &self,
        process: &mut EngineProcess,
        fen: &str,
        depth: u32,
        multipv: u32,
        on_update: &mut F,
    ) -> AppResult<EngineUpdate>
    where
        F: FnMut(&EngineUpdate),
    {
        process.set_multipv(multipv).await?;
        process.set_position(fen).await?;
        process.go_depth(depth).await?;
        let mut lines: BTreeMap<u32, EngineLine> = BTreeMap::new();
        let mut update = EngineUpdate {
            fen: fen.to_string(),
            depth: 0,
            seldepth: None,
            nodes: None,
            nps: None,
            lines: Vec::new(),
            best_move: None,
            best_san: None,
            finished: false,
        };
        let mut last_emit = Instant::now()
            .checked_sub(THROTTLE)
            .unwrap_or_else(Instant::now);
        let mut stop_sent = false;
        loop {
            if self.cancel.load(Ordering::SeqCst) && !stop_sent {
                process.stop().await?;
                stop_sent = true;
            }
            let line = process.read_line().await?;
            if let Some(info) = parse_info(&line) {
                let score = Score::new(info.cp, info.mate);
                let rank = info.multipv.max(1);
                lines.insert(
                    rank,
                    EngineLine {
                        rank,
                        cp: info.cp,
                        mate: info.mate,
                        win_prob: score.win_prob(),
                        uci_moves: info.pv.clone(),
                        san_moves: line_to_san(fen, &info.pv),
                    },
                );
                update.depth = info.depth.unwrap_or(update.depth);
                update.seldepth = info.seldepth.or(update.seldepth);
                update.nodes = info.nodes.or(update.nodes);
                update.nps = info.nps.or(update.nps);
                update.lines = lines.values().cloned().collect();
                if last_emit.elapsed() >= THROTTLE {
                    on_update(&update);
                    last_emit = Instant::now();
                }
            } else if line.starts_with("bestmove") {
                if stop_sent {
                    return Err(AppError::Engine(CANCELLED.to_string()));
                }
                update.best_move = parse_bestmove(&line);
                update.best_san = update
                    .best_move
                    .as_deref()
                    .and_then(|best| move_to_san(fen, best));
                update.finished = true;
                on_update(&update);
                return Ok(update);
            }
        }
    }

    pub async fn shutdown(&self) {
        let mut process = self.process.lock().await;
        process.shutdown().await;
    }
}

fn version_from_name(name: &str) -> String {
    name.rsplit_once(' ')
        .map(|(_, version)| version.trim().to_string())
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn is_cancel_error(err: &AppError) -> bool {
    matches!(err, AppError::Engine(message) if message.as_str() == CANCELLED)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "manager_tests.rs"]
mod tests;
