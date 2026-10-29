use tauri::ipc::Channel;

use crate::db::analyses_repo;
use crate::db::connection::Db;
use crate::db::games_repo;
use crate::domain::analysis::AnalysisProgress;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::GameDetail;
use crate::engine::EngineManager;
use crate::pgn::parse_single;
use crate::service::build::{build_analysis, BuildInput};
use crate::service::evaluation::evaluate_positions;

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

pub async fn analyze_game(
    db: &Db,
    engine: &EngineManager,
    game_id: i64,
    depth: u32,
    multipv: u32,
    progress: Option<&Channel<AnalysisProgress>>,
) -> AppResult<GameDetail> {
    // Одна задача на движок: отмена больше не сбрасывается каждой позицией.
    let _job = engine.begin_job().await?;
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
    let evaluations = evaluate_positions(engine, &fens, depth, multipv, |done, total| {
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
    .await?;
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
        engine_name: engine.name(),
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
