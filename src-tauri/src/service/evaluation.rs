use crate::domain::analysis::PvLine;
use crate::domain::error::AppResult;
use crate::engine::EngineManager;

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
        let update = engine.analyze(fen, depth, multipv, |_| {}).await?;
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
        evaluations.push(PositionEval {
            fen: fen.clone(),
            lines,
            best_move: update.best_move,
            depth: update.depth,
        });
        on_progress(index + 1, total);
    }
    Ok(evaluations)
}
