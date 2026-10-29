use rusqlite::Row;

use crate::domain::analysis::{
    CategoryCounts, GameAnalysis, MoveCategory, MoveEvaluation, Phase, PvLine, TimeFlag,
    DEFAULT_ENGINE_VERSION,
};
use crate::domain::game::Color;

pub(super) fn map_analysis(row: &Row<'_>) -> rusqlite::Result<GameAnalysis> {
    let counts_json: String = row.get("counts_json")?;
    let counts = serde_json::from_str::<CategoryCounts>(&counts_json)
        .unwrap_or_else(|_| CategoryCounts::new());
    Ok(GameAnalysis {
        game_id: row.get("game_id")?,
        engine_name: row.get("engine_name")?,
        // Старые записи могли появиться до миграции v4 с пустой версией.
        engine_version: row
            .get::<_, Option<String>>("engine_version")?
            .unwrap_or_else(|| DEFAULT_ENGINE_VERSION.to_string()),
        classifier_version: row.get("classifier_version")?,
        accuracy_method_version: row.get("accuracy_method_version")?,
        depth: row.get("depth")?,
        multipv: row.get("multipv")?,
        analyzed_at: row.get("analyzed_at")?,
        white_accuracy: row.get("white_accuracy")?,
        black_accuracy: row.get("black_accuracy")?,
        white_acpl: row.get("white_acpl")?,
        black_acpl: row.get("black_acpl")?,
        white_perf_elo: row.get("white_perf_elo")?,
        black_perf_elo: row.get("black_perf_elo")?,
        white_opening_acc: row.get("white_opening_acc")?,
        white_middlegame_acc: row.get("white_middlegame_acc")?,
        white_endgame_acc: row.get("white_endgame_acc")?,
        black_opening_acc: row.get("black_opening_acc")?,
        black_middlegame_acc: row.get("black_middlegame_acc")?,
        black_endgame_acc: row.get("black_endgame_acc")?,
        counts,
        summary_text: row.get("summary_text")?,
        key_moments_count: row.get("key_moments_count")?,
    })
}

pub(super) fn map_move(row: &Row<'_>) -> rusqlite::Result<MoveEvaluation> {
    let phase: String = row.get("phase")?;
    let classification: String = row.get("classification")?;
    let time_flag: String = row.get("time_flag")?;
    let player_color: String = row.get("player_color")?;
    let pv_json: String = row.get("pv_lines_json")?;
    let pv_lines = serde_json::from_str::<Vec<PvLine>>(&pv_json).unwrap_or_default();
    Ok(MoveEvaluation {
        game_id: row.get("game_id")?,
        ply: row.get("ply")?,
        player_color: Color::from_str_opt(&player_color).unwrap_or(Color::White),
        phase: Phase::from_str_opt(&phase).unwrap_or(Phase::Middlegame),
        fen_before: row.get("fen_before")?,
        fen_after: row.get("fen_after")?,
        played_uci: row.get("played_uci")?,
        played_san: row.get("played_san")?,
        best_uci: row.get("best_uci")?,
        best_san: row.get("best_san")?,
        eval_cp: row.get("eval_cp")?,
        eval_mate: row.get("eval_mate")?,
        best_eval_cp: row.get("best_eval_cp")?,
        best_eval_mate: row.get("best_eval_mate")?,
        win_prob_white: row.get("win_prob_white")?,
        win_prob_loss: row.get("win_prob_loss")?,
        move_accuracy: row.get("move_accuracy")?,
        classification: MoveCategory::from_str_opt(&classification).unwrap_or(MoveCategory::Good),
        is_critical: row.get::<_, i64>("is_critical")? != 0,
        is_key_moment: row.get::<_, i64>("is_key_moment")? != 0,
        material_diff_white: row.get("material_diff_white")?,
        clock_remaining_sec: row.get("clock_remaining_sec")?,
        time_spent_sec: row.get("time_spent_sec")?,
        time_flag: TimeFlag::from_str_opt(&time_flag).unwrap_or(TimeFlag::Normal),
        explanation_tag: row.get("explanation_tag")?,
        explanation_text: row.get("explanation_text")?,
        pv_lines,
        user_comment: row.get("user_comment")?,
    })
}
