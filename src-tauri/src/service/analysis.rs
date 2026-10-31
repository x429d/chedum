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
use crate::service::checkpoint::Checkpoint;
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

/// Who evaluates positions: a single process (a queued game) or the pool, which
/// spreads one game's positions across free processes.
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

/// Analyzes one game with the whole pool: positions are split into blocks and
/// evaluated in parallel. Makes sense when free processes outnumber games in
/// flight — otherwise the queue already occupies all cores.
///
/// The result does not match sequential analysis bit for bit: blocks start with
/// a clean table, so each position lacks the "warmed" history of previous
/// positions. But the evaluation does not depend on which process took a block
/// or in what order — that is, it is reproducible for any pool size.
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
                tracing::warn!("Could not reset analysis attempts for game #{game_id}: {err}");
            }
        }
        Err(err) => {
            let cancelled = evaluator.is_cancelled() || is_cancelled_error(err);
            if !cancelled && !matches!(err, AppError::Busy(_)) {
                if let Err(record) =
                    games_repo::record_analysis_failure(db, game_id, &err.to_string())
                {
                    tracing::warn!(
                        "Could not record the analysis error for game #{game_id}: {record}"
                    );
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
        "Parsing PGN and preparing positions",
    );
    let parsed = parse_single(&game.pgn)?;
    if parsed.moves.is_empty() {
        return Err(AppError::Validation("The game has no moves".to_string()));
    }
    let total_positions = parsed.moves.len() + 1;
    let mut fens = Vec::with_capacity(total_positions);
    fens.push(parsed.moves[0].fen_before.clone());
    for mv in &parsed.moves {
        fens.push(mv.fen_after.clone());
    }
    let total_plies = parsed.moves.len() as u32;
    let checkpoint = Checkpoint::new(
        db,
        game_id,
        depth,
        multipv,
        evaluator.name(),
        evaluator.version(),
    );
    checkpoint.prepare()?;
    let evaluations = match evaluator {
        Evaluator::Single(engine) => {
            // One job per engine: cancellation is no longer reset per position.
            let _job = engine.begin_job().await?;
            // Empty hash table before the game: ACPL must not depend on the
            // order in which games were processed.
            engine.prepare_game().await?;
            evaluate_positions(
                engine,
                &fens,
                depth,
                multipv,
                Some(&checkpoint),
                |done, total| {
                    let percent = 5.0 + 70.0 * (done as f64 / total as f64);
                    emit_progress(
                        progress,
                        "engine",
                        done as u32,
                        total as u32,
                        percent,
                        &format!("Stockfish: position {done} of {total}"),
                    );
                },
            )
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
                    &format!("Stockfish: position {done} of {total}"),
                );
            });
            evaluate_positions_pooled(
                pool,
                &fens,
                depth,
                multipv,
                Some(checkpoint.clone()),
                on_done,
            )
            .await?
        }
    };
    emit_progress(
        progress,
        "aggregate",
        total_plies,
        total_plies,
        82.0,
        "Classifying moves and computing metrics",
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
        "Saving analysis to the local database",
    );
    analyses_repo::insert_analysis_draft(db, &draft)?;
    if let Err(err) = checkpoint.clear() {
        tracing::warn!("Could not clear analysis checkpoints for game #{game_id}: {err}");
    }
    emit_progress(
        progress,
        "done",
        total_plies,
        total_plies,
        100.0,
        "Analysis complete",
    );
    Ok(GameDetail {
        game,
        analysis: Some(draft.analysis),
        moves: draft.moves,
    })
}
