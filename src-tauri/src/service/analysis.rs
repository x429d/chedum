use std::sync::Arc;

use tauri::ipc::Channel;

use crate::db::analyses_repo;
use crate::db::connection::Db;
use crate::db::games_repo;
use crate::domain::analysis::AnalysisProgress;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::GameDetail;
use crate::engine::{is_cancelled_error, EngineManager, EnginePool};
use crate::pgn::parse_single;
use crate::service::build::{build_analysis, BuildInput};
use crate::service::evaluation::{evaluate_positions, evaluate_positions_pooled};

pub fn emit_progress(
    channel: Option<&Channel<AnalysisProgress>>,
    stage: &str,
    ply: u32,
    total: u32,
    percent: f64,
    message: &str,
) {
    if let Some(channel) = channel {
        let _ = channel.send(AnalysisProgress {
            stage: stage.to_string(),
            ply,
            total,
            percent,
            message: message.to_string(),
        });
    }
}

/// Кто считает позиции: один процесс (партия в очереди) или пул, который
/// раскладывает позиции одной партии по свободным процессам.
enum Evaluator<'a> {
    Single(&'a EngineManager),
    Pooled(&'a Arc<EnginePool>),
}

impl Evaluator<'_> {
    fn is_cancelled(&self) -> bool {
        match self {
            Evaluator::Single(engine) => engine.is_cancelled(),
            Evaluator::Pooled(pool) => pool.is_stopped(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Evaluator::Single(engine) => engine.name(),
            Evaluator::Pooled(pool) => pool.name(),
        }
    }

    fn version(&self) -> &str {
        match self {
            Evaluator::Single(engine) => engine.version(),
            Evaluator::Pooled(pool) => pool.version(),
        }
    }
}

pub async fn analyze_game(
    db: &Db,
    engine: &EngineManager,
    game_id: i64,
    depth: u32,
    multipv: u32,
    progress: Option<&Channel<AnalysisProgress>>,
) -> AppResult<GameDetail> {
    analyze_game_with(
        db,
        Evaluator::Single(engine),
        game_id,
        depth,
        multipv,
        progress,
    )
    .await
}

/// Разбор одной партии силами всего пула: позиции режутся на блоки и
/// считаются параллельно. Имеет смысл, когда свободных процессов больше,
/// чем партий в работе, — иначе очередь и так занимает все ядра.
///
/// Результат не совпадает побитово с последовательным разбором: блоки стартуют
/// с чистой таблицей, поэтому у каждой позиции нет «прогретой» истории
/// предыдущих позиций. Зато оценка не зависит от того, какой процесс и в каком
/// порядке взял блок, — то есть воспроизводима при любом размере пула.
pub async fn analyze_game_parallel(
    db: &Db,
    pool: &Arc<EnginePool>,
    game_id: i64,
    depth: u32,
    multipv: u32,
    progress: Option<&Channel<AnalysisProgress>>,
) -> AppResult<GameDetail> {
    analyze_game_with(
        db,
        Evaluator::Pooled(pool),
        game_id,
        depth,
        multipv,
        progress,
    )
    .await
}

async fn analyze_game_with(
    db: &Db,
    evaluator: Evaluator<'_>,
    game_id: i64,
    depth: u32,
    multipv: u32,
    progress: Option<&Channel<AnalysisProgress>>,
) -> AppResult<GameDetail> {
    let outcome = analyze_game_inner(db, &evaluator, game_id, depth, multipv, progress).await;
    match &outcome {
        Ok(_) => {
            if let Err(err) = games_repo::reset_analysis_attempts(db, game_id) {
                eprintln!("Chenal: не удалось сбросить попытки анализа #{game_id}: {err}");
            }
        }
        Err(err) => {
            let cancelled = evaluator.is_cancelled() || is_cancelled_error(err);
            if !cancelled && !matches!(err, AppError::Busy(_)) {
                if let Err(record) =
                    games_repo::record_analysis_failure(db, game_id, &err.to_string())
                {
                    eprintln!("Chenal: не удалось записать ошибку анализа #{game_id}: {record}");
                }
            }
        }
    }
    outcome
}

async fn analyze_game_inner(
    db: &Db,
    evaluator: &Evaluator<'_>,
    game_id: i64,
    depth: u32,
    multipv: u32,
    progress: Option<&Channel<AnalysisProgress>>,
) -> AppResult<GameDetail> {
    let game = games_repo::get_game(db, game_id)?;
    emit_progress(
        progress,
        "prepare",
        0,
        0,
        2.0,
        "Разбор PGN и подготовка позиций",
    );
    let parsed = parse_single(&game.pgn)?;
    if parsed.moves.is_empty() {
        return Err(AppError::Validation("В партии нет ходов".to_string()));
    }
    let total_positions = parsed.moves.len() + 1;
    let mut fens = Vec::with_capacity(total_positions);
    fens.push(parsed.moves[0].fen_before.clone());
    for mv in &parsed.moves {
        fens.push(mv.fen_after.clone());
    }
    let total_plies = parsed.moves.len() as u32;
    let evaluations = match evaluator {
        Evaluator::Single(engine) => {
            // Одна задача на движок: отмена больше не сбрасывается каждой позицией.
            let _job = engine.begin_job().await?;
            // Пустая хеш-таблица перед партией: ACPL не зависит от порядка партий.
            engine.prepare_game().await?;
            evaluate_positions(engine, &fens, depth, multipv, |done, total| {
                let percent = 5.0 + 70.0 * (done as f64 / total as f64);
                emit_progress(
                    progress,
                    "engine",
                    done as u32,
                    total as u32,
                    percent,
                    &format!("Stockfish: позиция {done} из {total}"),
                );
            })
            .await?
        }
        Evaluator::Pooled(pool) => {
            let total = fens.len();
            let channel = progress.cloned();
            let on_done: Arc<dyn Fn(usize) + Send + Sync> = Arc::new(move |done| {
                let percent = 5.0 + 70.0 * (done as f64 / total as f64);
                emit_progress(
                    channel.as_ref(),
                    "engine",
                    done as u32,
                    total as u32,
                    percent,
                    &format!("Stockfish: позиция {done} из {total}"),
                );
            });
            evaluate_positions_pooled(pool, &fens, depth, multipv, on_done).await?
        }
    };
    emit_progress(
        progress,
        "aggregate",
        total_plies,
        total_plies,
        82.0,
        "Классификация ходов и расчёт метрик",
    );
    let draft = build_analysis(&BuildInput {
        game: &game,
        parsed: &parsed,
        evaluations: &evaluations,
        engine_name: evaluator.name(),
        engine_version: evaluator.version(),
        depth,
        multipv,
    })?;
    emit_progress(
        progress,
        "save",
        total_plies,
        total_plies,
        92.0,
        "Сохранение анализа в локальную базу",
    );
    analyses_repo::insert_analysis_draft(db, &draft)?;
    emit_progress(
        progress,
        "done",
        total_plies,
        total_plies,
        100.0,
        "Анализ завершён",
    );
    Ok(GameDetail {
        game,
        analysis: Some(draft.analysis),
        moves: draft.moves,
    })
}
