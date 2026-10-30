use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use crate::domain::analysis::PvLine;
use crate::domain::error::{AppError, AppResult};
use crate::engine::{EngineManager, EnginePool, EngineUpdate};

#[derive(Debug, Clone)]
pub struct PositionEval {
    pub fen: String,
    pub lines: Vec<PvLine>,
    pub best_move: Option<String>,
    pub depth: u32,
}

/// Последовательно оценивает позиции, сохраняя MultiPV-линии.
pub async fn evaluate_positions<F>(
    engine: &EngineManager,
    fens: &[String],
    depth: u32,
    multipv: u32,
    mut on_progress: F,
) -> AppResult<Vec<PositionEval>>
where
    F: FnMut(usize, usize),
{
    let total = fens.len();
    let mut evaluations = Vec::with_capacity(total);
    for (index, fen) in fens.iter().enumerate() {
        let update = evaluate_position(engine, fen, depth, multipv).await?;
        evaluations.push(position_eval(fen, update));
        on_progress(index + 1, total);
    }
    Ok(evaluations)
}

/// Размер блока при параллельной оценке. Блок фиксирован и не зависит от
/// размера пула, поэтому и разбиение, и результат каждой позиции
/// воспроизводимы при любом числе процессов. На малой глубине поиск дешёвый
/// и блоки крупнее (меньше холодных стартов), на большой — мельче, чтобы
/// медленные позиции не растягивали хвост.
fn block_size(depth: u32) -> usize {
    if depth >= 18 {
        4
    } else {
        8
    }
}

/// Оценивает позиции блоками, распределяя блоки по пулу процессов.
/// Внутри блока transposition table переходит от позиции к позиции — так же,
/// как при последовательном разборе, — но каждый блок начинается с чистого
/// листа. Иначе оценка зависела бы от того, какие позиции успел посчитать
/// конкретный процесс, а значит и от числа воркеров.
pub async fn evaluate_positions_pooled(
    pool: &Arc<EnginePool>,
    fens: &[String],
    depth: u32,
    multipv: u32,
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
                if pool.is_stopped() {
                    return Err(crate::engine::cancelled_error());
                }
                let lease = pool.acquire().await;
                let engine = lease.manager();
                // Задача на весь движок: параллельный блок не должен
                // дождаться отмены, которая уже пришла в пул.
                if pool.is_stopped() {
                    return Err(crate::engine::cancelled_error());
                }
                let _job = engine.begin_job().await?;
                // Чистая таблица на блок: только от неё зависит результат.
                engine.prepare_game().await?;
                let start = block * size;
                let end = (start + size).min(fens.len());
                for index in start..end {
                    let update = evaluate_position(engine, &fens[index], depth, multipv).await?;
                    results.push((index, position_eval(&fens[index], update)));
                    let finished = done.fetch_add(1, Ordering::SeqCst) + 1;
                    on_done(finished);
                }
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
                // Один сломанный блок отменяет остальные: ждать их бессмысленно.
                aborted.store(true, Ordering::SeqCst);
                if failure.is_none() {
                    failure = Some(err);
                }
            }
            Err(err) => {
                aborted.store(true, Ordering::SeqCst);
                if failure.is_none() {
                    failure = Some(AppError::Internal(format!(
                        "Задача анализа прервана: {err}"
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
                AppError::Internal(format!("Позиция {} осталась без оценки", index + 1))
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

/// Одна позиция с одним повтором: менеджер уже перезапустил движок,
/// поэтому повтор после сбоя обычно проходит и партия не теряется.
async fn evaluate_position(
    engine: &EngineManager,
    fen: &str,
    depth: u32,
    multipv: u32,
) -> AppResult<EngineUpdate> {
    match engine.analyze(fen, depth, multipv, |_| {}).await {
        Ok(update) => Ok(update),
        Err(first) if !engine.is_cancelled() => {
            eprintln!("Chenal: повтор оценки позиции после сбоя движка: {first}");
            engine.analyze(fen, depth, multipv, |_| {}).await
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::path::Path;

    use super::*;

    fn bundled_engine() -> Option<std::path::PathBuf> {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin");
        path.is_file().then_some(path)
    }

    /// Одинаковые позиции на пулах разного размера: блоки стартуют с чистой
    /// таблицей, поэтому число процессов не должно менять ни порядок, ни оценки.
    #[tokio::test]
    async fn pooled_evaluation_is_order_and_pool_size_independent() {
        let Some(path) = bundled_engine() else {
            return;
        };
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
            "r1bqkbnr/pp1ppppp/2n5/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            "r1bqkbnr/pp1ppppp/2n5/2p5/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
            "r1bqkbnr/pp2pppp/2n5/2pp4/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
            "r1bqkbnr/pp2pppp/2n5/3p4/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4",
            "r1bqkbnr/pp2pppp/2n5/3P4/2B5/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4",
            "r1bqkbnr/pp3ppp/2n5/3p4/2B5/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 5",
        ]
        .map(String::from);
        let small = EnginePool::spawn(&path, 2, 16).await.unwrap();
        let large = EnginePool::spawn(&path, 3, 16).await.unwrap();
        let first = evaluate_positions_pooled(&small, &fens, 6, 1, Arc::new(|_| {}))
            .await
            .unwrap();
        let second = evaluate_positions_pooled(&large, &fens, 6, 1, Arc::new(|_| {}))
            .await
            .unwrap();
        small.shutdown().await;
        large.shutdown().await;

        assert_eq!(first.len(), fens.len());
        for (index, evaluation) in first.iter().enumerate() {
            assert_eq!(evaluation.fen, fens[index], "порядок позиций");
            let other = &second[index];
            assert_eq!(evaluation.best_move, other.best_move, "позиция {index}");
            assert_eq!(
                evaluation.lines.first().map(|line| line.score_cp),
                other.lines.first().map(|line| line.score_cp),
                "позиция {index}"
            );
        }
    }

    #[tokio::test]
    async fn stop_cancels_parallel_blocks() {
        let Some(path) = bundled_engine() else {
            return;
        };
        let fens: Vec<String> = (0..16)
            .map(|_| "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
            .collect();
        let pool = EnginePool::spawn(&path, 2, 16).await.unwrap();
        let stopper = {
            let pool = Arc::clone(&pool);
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(60)).await;
                pool.request_stop();
            })
        };
        let result = evaluate_positions_pooled(&pool, &fens, 18, 1, Arc::new(|_| {})).await;
        stopper.await.unwrap();
        pool.shutdown().await;
        assert!(result.is_err(), "остановка должна прерывать блоки");
    }
}
