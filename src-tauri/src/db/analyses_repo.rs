use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::connection::Db;
use crate::domain::analysis::{
    AnalysisDraft, CategoryCounts, GameAnalysis, MoveCategory, MoveEvaluation, Phase, PvLine,
    TimeFlag,
};
use crate::domain::error::{AppError, AppResult};

const ANALYSIS_UPSERT: &str = "INSERT INTO game_analyses (
        game_id, engine_name, depth, multipv, analyzed_at, white_accuracy, black_accuracy,
        white_acpl, black_acpl, white_perf_elo, black_perf_elo, white_opening_acc,
        white_middlegame_acc, white_endgame_acc, black_opening_acc, black_middlegame_acc,
        black_endgame_acc, counts_json, summary_text, key_moments_count
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
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
        key_moments_count = excluded.key_moments_count";

pub fn insert_analysis_draft(db: &Db, draft: &AnalysisDraft) -> AppResult<()> {
    db.with_tx(|tx| insert_analysis_conn(tx, draft))
}

/// Вставляет анализ и ходы внутри уже открытой транзакции.
pub fn insert_analysis_conn(conn: &Connection, draft: &AnalysisDraft) -> AppResult<()> {
    let analysis = &draft.analysis;
    let counts_json = serde_json::to_string(&analysis.counts)?;
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
    conn.execute(
        "INSERT INTO move_evaluations (
            game_id, ply, phase, fen_before, fen_after, played_uci, played_san, best_uci, best_san,
            eval_cp, eval_mate, best_eval_cp, best_eval_mate, win_prob_white, win_prob_loss,
            move_accuracy, classification, is_critical, material_diff_white, clock_remaining_sec,
            time_spent_sec, time_flag, explanation_tag, explanation_text, pv_lines_json, user_comment
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
        params![
            mv.game_id,
            mv.ply,
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
            mv.material_diff_white,
            mv.clock_remaining_sec,
            mv.time_spent_sec,
            mv.time_flag.as_str(),
            mv.explanation_tag,
            mv.explanation_text,
            pv_json,
            mv.user_comment,
        ],
    )?;
    Ok(())
}

pub fn get_analysis(db: &Db, game_id: i64) -> AppResult<Option<GameAnalysis>> {
    db.with(|conn| {
        conn.query_row(
            "SELECT * FROM game_analyses WHERE game_id = ?1",
            params![game_id],
            map_analysis,
        )
        .optional()
        .map_err(AppError::from)
    })
}

pub fn get_moves(db: &Db, game_id: i64) -> AppResult<Vec<MoveEvaluation>> {
    db.with(|conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC")?;
        let rows = stmt.query_map(params![game_id], map_move)?;
        let mut moves = Vec::new();
        for row in rows {
            moves.push(row?);
        }
        Ok(moves)
    })
}

pub fn save_move_comment(db: &Db, game_id: i64, ply: i32, comment: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE move_evaluations SET user_comment = ?1 WHERE game_id = ?2 AND ply = ?3",
            params![comment, game_id, ply],
        )?;
        Ok(())
    })
}

pub fn has_analysis(db: &Db, game_id: i64) -> AppResult<bool> {
    db.with(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM game_analyses WHERE game_id = ?1",
            params![game_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    })
}

fn map_analysis(row: &Row<'_>) -> rusqlite::Result<GameAnalysis> {
    let counts_json: String = row.get("counts_json")?;
    let counts = serde_json::from_str::<CategoryCounts>(&counts_json)
        .unwrap_or_else(|_| CategoryCounts::new());
    Ok(GameAnalysis {
        game_id: row.get("game_id")?,
        engine_name: row.get("engine_name")?,
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

fn map_move(row: &Row<'_>) -> rusqlite::Result<MoveEvaluation> {
    let phase: String = row.get("phase")?;
    let classification: String = row.get("classification")?;
    let time_flag: String = row.get("time_flag")?;
    let pv_json: String = row.get("pv_lines_json")?;
    let pv_lines = serde_json::from_str::<Vec<PvLine>>(&pv_json).unwrap_or_default();
    Ok(MoveEvaluation {
        game_id: row.get("game_id")?,
        ply: row.get("ply")?,
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
