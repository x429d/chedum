use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::analytics::{
    game_accuracy, performance_elo, phase_accuracy, summary_from_analysis, win_prob_cp,
    win_prob_mate, SummaryContext,
};
use crate::db::connection::Db;
use crate::domain::analysis::{
    AnalysisDraft, CategoryCounts, GameAnalysis, MoveCategory, MoveEvaluation, Phase, PvLine,
    TimeFlag,
};
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{Color, UserColor};

/// Ключ конфигурации анализа: агрегаты хаба смешивают только одинаковые.
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

/// Вставляет анализ и ходы внутри уже открытой транзакции.
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
    conn.execute(
        "INSERT INTO move_evaluations (
            game_id, ply, phase, fen_before, fen_after, played_uci, played_san, best_uci, best_san,
            eval_cp, eval_mate, best_eval_cp, best_eval_mate, win_prob_white, win_prob_loss,
            move_accuracy, classification, is_critical, is_key_moment, material_diff_white, clock_remaining_sec,
            time_spent_sec, time_flag, explanation_tag, explanation_text, pv_lines_json, user_comment
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
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
            i32::from(mv.is_key_moment),
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
        let analysis = conn
            .query_row(
                "SELECT * FROM game_analyses WHERE game_id = ?1",
                params![game_id],
                map_analysis,
            )
            .optional()?;
        match analysis {
            Some(mut analysis) => {
                // Текст резюме всегда собирается из тех же полей, что и карточки,
                // поэтому старые записи не могут показать другое число точности.
                analysis.summary_text = summary_for_conn(conn, game_id, &analysis)?;
                Ok(Some(analysis))
            }
            None => Ok(None),
        }
    })
}

/// Поля партии и счётчики тайм-флагов, нужные для резюме.
fn summary_for_conn(
    conn: &Connection,
    game_id: i64,
    analysis: &GameAnalysis,
) -> AppResult<String> {
    let (white, black, result, user_color, eco, opening): (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn.query_row(
        "SELECT white_username, black_username, result, user_color, eco, opening_name
         FROM games WHERE id = ?1",
        params![game_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;
    let (time_trouble, impulsive) = user_time_flags(conn, game_id, &user_color)?;
    let context = SummaryContext {
        white: &white,
        black: &black,
        result: &result,
        user_color: UserColor::from_str_opt(&user_color).unwrap_or(UserColor::None),
        eco: eco.as_deref(),
        opening: opening.as_deref(),
    };
    Ok(summary_from_analysis(
        &context,
        analysis,
        time_trouble,
        impulsive,
    ))
}

fn user_time_flags(conn: &Connection, game_id: i64, user_color: &str) -> AppResult<(u32, u32)> {
    let parity = if user_color == "black" { 0 } else { 1 };
    conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN time_flag = 'time_trouble' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN time_flag = 'impulsive_error' THEN 1 ELSE 0 END), 0)
         FROM move_evaluations WHERE game_id = ?1 AND ply % 2 = ?2",
        params![game_id, parity],
        |row| Ok((row.get::<_, i64>(0)? as u32, row.get::<_, i64>(1)? as u32)),
    )
    .map_err(AppError::from)
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
        engine_version: row.get("engine_version")?,
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

const ACCURACY_ALGO_KEY: &str = "accuracy_algo";
const SUMMARY_VERSION_KEY: &str = "summary_text_version";
const SUMMARY_VERSION: &str = "single-source-v2";
const KEY_MOMENTS_VERSION_KEY: &str = "key_moments_version";
const KEY_MOMENTS_VERSION: &str = "errors-gap3-v2";

/// Пересчитывает ключевые моменты по сохранённым оценкам. Движок не нужен:
/// после смены алгоритма отбора достаточно пересмотреть ходы.
pub fn backfill_key_moments(db: &Db) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [KEY_MOMENTS_VERSION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(KEY_MOMENTS_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            let user_color: String = tx.query_row(
                "SELECT user_color FROM games WHERE id = ?1",
                [game_id],
                |row| row.get(0),
            )?;
            let moves = load_moves_conn(tx, game_id)?;
            let selected = crate::analytics::select_key_moments(
                &moves,
                Color::from_str_opt(&user_color),
                crate::analytics::moments::player_from_fen,
            );
            for (index, mv) in moves.iter().enumerate() {
                tx.execute(
                    "UPDATE move_evaluations SET is_key_moment = ?2
                     WHERE game_id = ?1 AND ply = ?3",
                    params![game_id, i32::from(selected.contains(&index)), mv.ply],
                )?;
            }
            tx.execute(
                "UPDATE game_analyses SET key_moments_count = ?2 WHERE game_id = ?1",
                params![game_id, selected.len() as i32],
            )?;
            let analysis = tx.query_row(
                "SELECT * FROM game_analyses WHERE game_id = ?1",
                params![game_id],
                map_analysis,
            )?;
            let summary = summary_for_conn(tx, game_id, &analysis)?;
            tx.execute(
                "UPDATE game_analyses SET summary_text = ?2 WHERE game_id = ?1",
                params![game_id, summary],
            )?;
            updated += 1;
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![KEY_MOMENTS_VERSION_KEY, KEY_MOMENTS_VERSION],
        )?;
        Ok(updated)
    })
}

fn load_moves_conn(conn: &Connection, game_id: i64) -> AppResult<Vec<MoveEvaluation>> {
    let mut stmt =
        conn.prepare("SELECT * FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC")?;
    let rows = stmt.query_map(params![game_id], map_move)?;
    let mut moves = Vec::new();
    for row in rows {
        moves.push(row?);
    }
    Ok(moves)
}

/// Пересобирает сохранённый текст резюме из сохранённых метрик анализа.
/// Нужен один раз после перехода на единый источник чисел; движок не требуется.
pub fn backfill_summaries(db: &Db) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [SUMMARY_VERSION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(SUMMARY_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            let analysis = tx.query_row(
                "SELECT * FROM game_analyses WHERE game_id = ?1",
                params![game_id],
                map_analysis,
            )?;
            let summary = summary_for_conn(tx, game_id, &analysis)?;
            tx.execute(
                "UPDATE game_analyses SET summary_text = ?2 WHERE game_id = ?1",
                params![game_id, summary],
            )?;
            updated += 1;
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![SUMMARY_VERSION_KEY, SUMMARY_VERSION],
        )?;
        Ok(updated)
    })
}

/// Сохранённый ход для пересчёта: полуход, Win% белых после, оценка до хода, фаза.
type StoredMoveRow = (i32, f64, Option<i32>, Option<i32>, Phase);

/// Пересчитывает точность по методу Lichess из уже сохранённых оценок.
/// Нужен один раз после смены алгоритма: движок и повторный анализ не требуются.
/// `force` — пересчитать даже если версия алгоритма уже отмечена (например, после
/// повторной загрузки демо-партий).
pub fn backfill_lichess_accuracy(db: &Db, force: bool) -> AppResult<usize> {
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [ACCURACY_ALGO_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if !force && current.as_deref() == Some(crate::domain::analysis::ACCURACY_METHOD_VERSION) {
            return Ok(0);
        }
        let game_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT game_id FROM game_analyses ORDER BY game_id")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<i64>>>()?
        };
        let mut updated = 0;
        for game_id in game_ids {
            if recompute_accuracy(tx, game_id)? {
                updated += 1;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![ACCURACY_ALGO_KEY, crate::domain::analysis::ACCURACY_METHOD_VERSION],
        )?;
        Ok(updated)
    })
}

fn recompute_accuracy(conn: &Connection, game_id: i64) -> AppResult<bool> {
    let (white_acpl, black_acpl, white_elo, black_elo) = conn.query_row(
        "SELECT a.white_acpl, a.black_acpl, g.white_elo, g.black_elo
         FROM game_analyses a JOIN games g ON g.id = a.game_id WHERE a.game_id = ?1",
        [game_id],
        |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?,
                row.get::<_, Option<f64>>(1)?,
                row.get::<_, Option<i32>>(2)?,
                row.get::<_, Option<i32>>(3)?,
            ))
        },
    )?;
    let rows: Vec<StoredMoveRow> = {
        let mut stmt = conn.prepare(
            "SELECT ply, win_prob_white, best_eval_cp, best_eval_mate, phase
             FROM move_evaluations WHERE game_id = ?1 ORDER BY ply ASC",
        )?;
        let mapped = stmt.query_map([game_id], |row| {
            let phase: String = row.get(4)?;
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                Phase::from_str_opt(&phase).unwrap_or(Phase::Middlegame),
            ))
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()?
    };
    if rows.is_empty() {
        return Ok(false);
    }

    let mut samples = Vec::with_capacity(rows.len());
    let mut phase_samples: [Vec<(f64, f64)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut phase_first: [Option<Color>; 3] = [None, None, None];
    let mut first_color = Color::White;
    for (index, (ply, win_white, cp, mate, phase)) in rows.iter().enumerate() {
        let player = if ply % 2 == 1 {
            Color::White
        } else {
            Color::Black
        };
        let before_white = score_win_prob(*cp, *mate);
        let (before, after) = if matches!(player, Color::White) {
            (before_white, *win_white)
        } else {
            (100.0 - before_white, 100.0 - *win_white)
        };
        let sample = (before, after);
        samples.push(sample);
        if index == 0 {
            first_color = player;
        }
        let slot = match phase {
            Phase::Opening => 0,
            Phase::Middlegame => 1,
            Phase::Endgame => 2,
        };
        if phase_first[slot].is_none() {
            phase_first[slot] = Some(player);
        }
        phase_samples[slot].push(sample);
    }

    let [white_accuracy, black_accuracy] = game_accuracy(&samples, first_color);
    let [white_phases, black_phases] = phase_accuracy(&phase_samples, &phase_first);
    conn.execute(
        "UPDATE game_analyses SET
            white_accuracy = ?2, black_accuracy = ?3,
            white_opening_acc = ?4, white_middlegame_acc = ?5, white_endgame_acc = ?6,
            black_opening_acc = ?7, black_middlegame_acc = ?8, black_endgame_acc = ?9,
            white_perf_elo = ?10, black_perf_elo = ?11
         WHERE game_id = ?1",
        params![
            game_id,
            white_accuracy,
            black_accuracy,
            white_phases[0],
            white_phases[1],
            white_phases[2],
            black_phases[0],
            black_phases[1],
            black_phases[2],
            performance_elo(white_elo, white_accuracy, white_acpl),
            performance_elo(black_elo, black_accuracy, black_acpl),
        ],
    )?;
    // Резюме хранится рядом с точностью и должно пересчитываться вместе с ней.
    let analysis = conn.query_row(
        "SELECT * FROM game_analyses WHERE game_id = ?1",
        params![game_id],
        map_analysis,
    )?;
    let summary = summary_for_conn(conn, game_id, &analysis)?;
    conn.execute(
        "UPDATE game_analyses SET summary_text = ?2 WHERE game_id = ?1",
        params![game_id, summary],
    )?;
    Ok(true)
}

fn score_win_prob(cp: Option<i32>, mate: Option<i32>) -> f64 {
    match mate {
        Some(mate) => win_prob_mate(mate),
        None => win_prob_cp(f64::from(cp.unwrap_or(0))),
    }
}
