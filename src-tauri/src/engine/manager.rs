use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use crate::analytics::Score;
use crate::domain::error::{AppError, AppResult};
use crate::engine::process::EngineProcess;
use crate::engine::pv::{line_to_san, move_to_san};
use crate::engine::types::{EngineLine, EngineUpdate};
use crate::engine::uci::{parse_bestmove, parse_info};

const THROTTLE: Duration = Duration::from_millis(75);

pub struct EngineManager {
    process: Arc<Mutex<EngineProcess>>,
    cancel: Arc<AtomicBool>,
    name: String,
}

impl EngineManager {
    pub async fn spawn(path: &Path) -> AppResult<Self> {
        let process = EngineProcess::spawn(path).await?;
        let name = process.name().to_string();
        Ok(EngineManager {
            process: Arc::new(Mutex::new(process)),
            cancel: Arc::new(AtomicBool::new(false)),
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    /// Анализирует позицию, транслируя обновления с троттлингом 75 мс.
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
        self.cancel.store(false, Ordering::SeqCst);
        let mut process = self.process.lock().await;
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
                    return Err(AppError::Engine("Анализ отменён".to_string()));
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[tokio::test]
    async fn analyzes_start_position_with_bundled_stockfish() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin");
        if !path.is_file() {
            return;
        }
        let manager = EngineManager::spawn(&path).await.unwrap();
        let mut updates = 0u32;
        let update = manager
            .analyze(START, 8, 2, |_| updates += 1)
            .await
            .unwrap();
        assert!(update.finished);
        assert!(!update.lines.is_empty());
        assert!(update.best_move.is_some());
        assert!(updates > 0);
        assert!(update.depth >= 8);
        manager.shutdown().await;
    }
}
