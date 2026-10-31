use rusqlite::{params, Connection};

use crate::db::connection::Db;
use crate::domain::analysis::{AnalysisDraft, GameAnalysis, MoveEvaluation};
use crate::domain::error::AppResult;

/// Analysis configuration key: hub aggregates only mix identical ones.
pub fn analysis_config_key(analysis: &GameAnalysis) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        analysis.engine_name,
        analysis.engine_version,
        analysis.depth,
        analysis.multipv,
        analysis.classifier_version,
        analysis.accuracy_method_version
    )
}

const ANALYSIS_UPSERT: &str = "INSERT INTO game_analyses (
        game_id, engine_name, depth, multipv, analyzed_at, white_accuracy, black_accuracy,
        white_acpl, black_acpl, white_perf_elo, black_perf_elo, white_opening_acc,
        white_middlegame_acc, white_endgame_acc, black_opening_acc, black_middlegame_acc,
        black_endgame_acc, counts_json, summary_text, key_moments_count,
        engine_version, classifier_version, accuracy_method_version, config_key
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
        ?21, ?22, ?23, ?24)
    ON CONFLICT(game_id) DO UPDATE SET
        engine_name = excluded.engine_name,
        depth = excluded.depth,
        multipv = excluded.multipv,
        analyzed_at = excluded.analyzed_at,
        white_accuracy = excluded.white_accuracy,
        black_accuracy = excluded.black_accuracy,
        white_acpl = excluded.white_acpl,
        black_acpl = excluded.black_acpl,
        white_perf_elo = excluded.white_perf_elo,
        black_perf_elo = excluded.black_perf_elo,
        white_opening_acc = excluded.white_opening_acc,
        white_middlegame_acc = excluded.white_middlegame_acc,
        white_endgame_acc = excluded.white_endgame_acc,
        black_opening_acc = excluded.black_opening_acc,
        black_middlegame_acc = excluded.black_middlegame_acc,
        black_endgame_acc = excluded.black_endgame_acc,
        counts_json = excluded.counts_json,
        summary_text = excluded.summary_text,
        key_moments_count = excluded.key_moments_count,
        engine_version = excluded.engine_version,
        classifier_version = excluded.classifier_version,
        accuracy_method_version = excluded.accuracy_method_version,
        config_key = excluded.config_key";

pub fn insert_analysis_draft(db: &Db, draft: &AnalysisDraft) -> AppResult<()> {
    db.with_tx(|tx| insert_analysis_conn(tx, draft))
}

/// Inserts the analysis and its moves inside an already open transaction.
pub fn insert_analysis_conn(conn: &Connection, draft: &AnalysisDraft) -> AppResult<()> {
    let analysis = &draft.analysis;
    let counts_json = serde_json::to_string(&analysis.counts)?;
    let config_key = analysis_config_key(analysis);
    conn.execute(
        ANALYSIS_UPSERT,
        params![
            analysis.game_id,
            analysis.engine_name,
            analysis.depth,
            analysis.multipv,
            analysis.analyzed_at,
            analysis.white_accuracy,
            analysis.black_accuracy,
            analysis.white_acpl,
            analysis.black_acpl,
            analysis.white_perf_elo,
            analysis.black_perf_elo,
            analysis.white_opening_acc,
            analysis.white_middlegame_acc,
            analysis.white_endgame_acc,
            analysis.black_opening_acc,
            analysis.black_middlegame_acc,
            analysis.black_endgame_acc,
            counts_json,
            analysis.summary_text,
            analysis.key_moments_count,
            analysis.engine_version,
            analysis.classifier_version,
            analysis.accuracy_method_version,
            config_key,
        ],
    )?;
    conn.execute(
        "DELETE FROM move_evaluations WHERE game_id = ?1",
        params![analysis.game_id],
    )?;
    for mv in &draft.moves {
        insert_move(conn, mv)?;
    }
    Ok(())
}

fn insert_move(conn: &Connection, mv: &MoveEvaluation) -> AppResult<()> {
    let pv_json = serde_json::to_string(&mv.pv_lines)?;
    let explanation_params_json = mv
        .explanation_params
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    conn.execute(
        "INSERT INTO move_evaluations (
            game_id, ply, player_color, phase, fen_before, fen_after, played_uci, played_san,
            best_uci, best_san, eval_cp, eval_mate, best_eval_cp, best_eval_mate, win_prob_white,
            win_prob_loss, move_accuracy, classification, is_critical, is_key_moment,
            material_diff_white, clock_remaining_sec, time_spent_sec, time_flag, explanation_tag,
            explanation_params_json, explanation_text, pv_lines_json, user_comment
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
        params![
            mv.game_id,
            mv.ply,
            mv.player_color.as_str(),
            mv.phase.as_str(),
            mv.fen_before,
            mv.fen_after,
            mv.played_uci,
            mv.played_san,
            mv.best_uci,
            mv.best_san,
            mv.eval_cp,
            mv.eval_mate,
            mv.best_eval_cp,
            mv.best_eval_mate,
            mv.win_prob_white,
            mv.win_prob_loss,
            mv.move_accuracy,
            mv.classification.as_str(),
            i32::from(mv.is_critical),
            i32::from(mv.is_key_moment),
            mv.material_diff_white,
            mv.clock_remaining_sec,
            mv.time_spent_sec,
            mv.time_flag.as_str(),
            mv.explanation_tag,
            explanation_params_json,
            mv.explanation_text,
            pv_json,
            mv.user_comment,
        ],
    )?;
    Ok(())
}
