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
const CANCELLED: &str = "Analysis cancelled";
/// How long a new job waits for the engine to be released before getting `busy`.
/// A short pause covers stopping live analysis; a long wait means the engine is
/// busy and should return an error.
const JOB_WAIT: Duration = Duration::from_secs(2);

/// Right to use the engine. Held for the whole game analysis, so parallel
/// batches cannot silently queue up indefinitely.
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

    /// One pool process: usually `threads = 1` and a moderate `Hash`.
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

    /// Version from `id name Stockfish 19` — "19". Stored in the analysis record.
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Starts a new analysis job: one job per engine.
    /// The cancel flag is reset only here, not on every position.
    pub async fn begin_job(&self) -> AppResult<EngineJob<'_>> {
        let Ok(slot) = timeout(JOB_WAIT, self.slot.lock()).await else {
            return Err(AppError::Busy(
                "The engine is already analyzing another game".to_string(),
            ));
        };
        self.cancel.store(false, Ordering::SeqCst);
        Ok(EngineJob { _slot: slot })
    }

    /// Starts a game from a clean engine state. Without `ucinewgame`, the
    /// transposition table carries over from game to game, and ACPL depends on
    /// processing order in the pool.
    pub async fn prepare_game(&self) -> AppResult<()> {
        let mut process = self.process.lock().await;
        if !process.is_running() {
            *process = EngineProcess::spawn(&self.path).await?;
        }
        let outcome = process.new_game().await;
        if outcome.is_err() {
            // A desynchronized UCI stream cannot be reused.
            if let Ok(fresh) = EngineProcess::spawn(&self.path).await {
                *process = fresh;
            }
        }
        outcome
    }

    /// Analyzes a position, streaming updates throttled to 75 ms.
    /// Must be called only inside a job opened by `begin_job`.
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
                // A desynchronized or stuck UCI stream cannot be reused:
                // the next call would start a search on top of the previous one.
                match EngineProcess::spawn(&self.path).await {
                    Ok(fresh) => *process = fresh,
                    Err(restart) => {
                        tracing::warn!("Could not restart Stockfish: {restart}");
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
                        san_moves: Vec::new(),
                    },
                );
                update.depth = info.depth.unwrap_or(update.depth);
                update.seldepth = info.seldepth.or(update.seldepth);
                update.nodes = info.nodes.or(update.nodes);
                update.nps = info.nps.or(update.nps);
                if last_emit.elapsed() >= THROTTLE {
                    fill_lines(&mut update, &lines, fen);
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
                fill_lines(&mut update, &lines, fen);
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

/// Moves the found lines into `update`, converting UCI variations to SAN.
/// SAN is computed only at emit time: batch analysis does not need updates at
/// all, and building them on every `info` line would re-parse the FEN and replay
/// the whole variation — dozens of times per position.
fn fill_lines(update: &mut EngineUpdate, lines: &BTreeMap<u32, EngineLine>, fen: &str) {
    update.lines = lines
        .values()
        .map(|line| {
            let mut line = line.clone();
            line.san_moves = line_to_san(fen, &line.uci_moves);
            line
        })
        .collect();
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

/// User cancellation is not a game failure: analysis attempts do not grow.
pub fn is_cancelled_error(err: &AppError) -> bool {
    is_cancel_error(err)
}

/// Cancellation error in the same shape the engine returns.
pub fn cancelled_error() -> AppError {
    AppError::Engine(CANCELLED.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "manager_tests.rs"]
mod tests;
