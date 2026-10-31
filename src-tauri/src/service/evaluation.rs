use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::analysis::PvLine;
use crate::domain::error::{AppError, AppResult};
use crate::engine::{EngineManager, EnginePool, EngineUpdate};
use crate::service::checkpoint::Checkpoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionEval {
    pub fen: String,
    pub lines: Vec<PvLine>,
    pub best_move: Option<String>,
    pub depth: u32,
}

/// Evaluates positions sequentially, reusing a checkpoint for already
/// completed positions.
pub async fn evaluate_positions<F>(
    engine: &EngineManager,
    fens: &[String],
    depth: u32,
    multipv: u32,
    checkpoint: Option<&Checkpoint>,
    mut on_progress: F,
) -> AppResult<Vec<PositionEval>>
where
    F: FnMut(usize, usize),
{
    let total = fens.len();
    let mut evaluations = Vec::with_capacity(total);
    for (index, fen) in fens.iter().enumerate() {
        if let Some(checkpoint) = checkpoint {
            if let Some(cached) = checkpoint.load_block(index)? {
                if let Some(evaluation) = cached.into_iter().next() {
                    evaluations.push(evaluation);
                    on_progress(index + 1, total);
                    continue;
                }
            }
        }
        let update = evaluate_position(engine, fen, depth, multipv).await?;
        let evaluation = position_eval(fen, update);
        if let Some(checkpoint) = checkpoint {
            if let Err(err) = checkpoint.store_block(index, std::slice::from_ref(&evaluation)) {
                tracing::warn!("Could not store analysis checkpoint at position {index}: {err}");
            }
        }
        evaluations.push(evaluation);
        on_progress(index + 1, total);
    }
    Ok(evaluations)
}

/// Fixed block size, independent of the pool size, so both the split and each
/// position's result are reproducible for any number of processes. Search is
/// cheap at low depth, so blocks are larger (fewer cold starts); at high depth
/// they are smaller so slow positions do not stretch the tail.
fn block_size(depth: u32) -> usize {
    if depth >= 18 {
        4
    } else {
        8
    }
}

/// Evaluates positions in blocks distributed across the process pool.
/// Inside a block the transposition table carries over between positions, just
/// like the sequential path, but every block starts with a clean table. That
/// makes the result independent of the worker count and scheduler. Completed
/// blocks are checkpointed and reused after a crash or cancellation.
pub async fn evaluate_positions_pooled(
    pool: &Arc<EnginePool>,
    fens: &[String],
    depth: u32,
    multipv: u32,
    checkpoint: Option<Checkpoint>,
    on_position_done: Arc<dyn Fn(usize) + Send + Sync>,
) -> AppResult<Vec<PositionEval>> {
    pool.clear_stop();
    let size = block_size(depth);
    let blocks = fens.len().div_ceil(size).max(1);
    let workers = pool.size().min(blocks).max(1);
    let next = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicUsize::new(0));
    let aborted = Arc::new(AtomicBool::new(false));
    let fens = Arc::new(fens.to_vec());
    let mut tasks = Vec::with_capacity(workers);
    for _ in 0..workers {
        let pool = Arc::clone(pool);
        let next = Arc::clone(&next);
        let done = Arc::clone(&done);
        let aborted = Arc::clone(&aborted);
        let fens = Arc::clone(&fens);
        let on_done = Arc::clone(&on_position_done);
        let checkpoint = checkpoint.clone();
        tasks.push(tokio::spawn(async move {
            let mut results: Vec<(usize, PositionEval)> = Vec::new();
            loop {
                if aborted.load(Ordering::SeqCst) {
                    return Ok(results);
                }
                let block = next.fetch_add(1, Ordering::SeqCst);
                if block >= blocks {
                    return Ok(results);
                }
                let start = block * size;
                let end = (start + size).min(fens.len());
                if let Some(checkpoint) = checkpoint.as_ref() {
                    if let Some(cached) = checkpoint.load_block(block)? {
                        if cached.len() == end - start {
                            for (offset, evaluation) in cached.into_iter().enumerate() {
                                results.push((start + offset, evaluation));
                                let finished = done.fetch_add(1, Ordering::SeqCst) + 1;
                                on_done(finished);
                            }
                            continue;
                        }
                    }
                }
                if pool.is_stopped() {
                    return Err(crate::engine::cancelled_error());
                }
                let lease = pool.acquire().await;
                let engine = lease.manager();
                if pool.is_stopped() {
                    return Err(crate::engine::cancelled_error());
                }
                let _job = engine.begin_job().await?;
                engine.prepare_game().await?;
                let mut block_results = Vec::with_capacity(end - start);
                for index in start..end {
                    let update = evaluate_position(engine, &fens[index], depth, multipv).await?;
                    block_results.push(position_eval(&fens[index], update));
                    let finished = done.fetch_add(1, Ordering::SeqCst) + 1;
                    on_done(finished);
                }
                if let Some(checkpoint) = checkpoint.as_ref() {
                    if let Err(err) = checkpoint.store_block(block, &block_results) {
                        tracing::warn!(
                            "Could not store analysis checkpoint for block {block}: {err}"
                        );
                    }
                }
                results.extend(
                    block_results
                        .into_iter()
                        .enumerate()
                        .map(|(offset, evaluation)| (start + offset, evaluation)),
                );
            }
        }));
    }

    let mut all: Vec<Option<PositionEval>> = vec![None; fens.len()];
    let mut failure: Option<AppError> = None;
    for task in tasks {
        match task.await {
            Ok(Ok(results)) => {
                for (index, evaluation) in results {
                    all[index] = Some(evaluation);
                }
            }
            Ok(Err(err)) => {
                // One broken block cancels the rest: waiting for them is pointless.
                aborted.store(true, Ordering::SeqCst);
                if failure.is_none() {
                    failure = Some(err);
                }
            }
            Err(err) => {
                aborted.store(true, Ordering::SeqCst);
                if failure.is_none() {
                    failure = Some(AppError::Internal(format!(
                        "Analysis task was interrupted: {err}"
                    )));
                }
            }
        }
    }
    if let Some(err) = failure {
        return Err(err);
    }
    all.into_iter()
        .enumerate()
        .map(|(index, evaluation)| {
            evaluation.ok_or_else(|| {
                AppError::Internal(format!("Position {} has no evaluation", index + 1))
            })
        })
        .collect()
}

fn position_eval(fen: &str, update: EngineUpdate) -> PositionEval {
    let lines = update
        .lines
        .iter()
        .map(|line| PvLine {
            rank: line.rank,
            score_cp: line.cp,
            score_mate: line.mate,
            win_prob: line.win_prob,
            uci_moves: line.uci_moves.clone(),
            san_moves: line.san_moves.clone(),
        })
        .collect();
    PositionEval {
        fen: fen.to_string(),
        lines,
        best_move: update.best_move,
        depth: update.depth,
    }
}

/// One position with one retry: the manager has already restarted the engine,
/// so a retry after a failure usually succeeds and the game is not lost.
async fn evaluate_position(
    engine: &EngineManager,
    fen: &str,
    depth: u32,
    multipv: u32,
) -> AppResult<EngineUpdate> {
    match engine.analyze(fen, depth, multipv, |_| {}).await {
        Ok(update) => Ok(update),
        Err(first) if !engine.is_cancelled() => {
            tracing::warn!("Retrying position evaluation after an engine failure: {first}");
            engine.analyze(fen, depth, multipv, |_| {}).await
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "evaluation_tests.rs"]
mod tests;
