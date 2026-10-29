use rusqlite::{params, Connection};

use crate::db::connection::Db;
use crate::db::stats_extra::{time_stats, trend};
use crate::domain::analysis::Phase;
use crate::domain::error::AppResult;
use crate::domain::stats::{ColorRecord, HubKpis, HubStats, OpeningStat, PhaseStat};

pub fn hub_stats(db: &Db) -> AppResult<HubStats> {
    db.with(|conn| {
        Ok(HubStats {
            kpis: kpis(conn)?,
            openings: openings(conn)?,
            phases: phases(conn)?,
            trend: trend(conn)?,
            time: time_stats(conn)?,
        })
    })
}

fn kpis(conn: &Connection) -> AppResult<HubKpis> {
    let row = conn.query_row(
        "SELECT
            COUNT(*),
            COALESCE(SUM(CASE WHEN a.game_id IS NOT NULL THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='1-0') OR (g.user_color='black' AND g.result='0-1') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result='1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='0-1') OR (g.user_color='black' AND g.result='1-0') THEN 1 ELSE 0 END), 0),
            AVG(CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_acpl ELSE a.black_acpl END)
         FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id
         WHERE g.user_color IN ('white', 'black')",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
            ))
        },
    )?;
    let total = row.0 as u32;
    let analyzed = row.1 as u32;
    let wins = row.2 as u32;
    let (avg_perf_elo, avg_perf_elo_spread) = performance_summary(conn, analyzed)?;
    Ok(HubKpis {
        total_games: total,
        analyzed_games: analyzed,
        wins,
        draws: row.3 as u32,
        losses: row.4 as u32,
        win_rate: if total > 0 {
            f64::from(wins) / f64::from(total) * 100.0
        } else {
            0.0
        },
        avg_accuracy: row.5,
        avg_acpl: row.6,
        avg_perf_elo,
        avg_perf_elo_spread,
        white: color_record(conn, "white")?,
        black: color_record(conn, "black")?,
    })
}

/// Оценка силы по точности показывается только на достаточной выборке
/// и всегда с разбросом: это экспериментальная величина, а не рейтинг.
const MIN_PERF_SAMPLE: u32 = 20;

fn performance_summary(conn: &Connection, analyzed: u32) -> AppResult<(Option<f64>, Option<f64>)> {
    if analyzed < MIN_PERF_SAMPLE {
        return Ok((None, None));
    }
    let mut stmt = conn.prepare(
        "SELECT CASE WHEN g.user_color='white' THEN a.white_perf_elo ELSE a.black_perf_elo END
         FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE g.user_color IN ('white', 'black')",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, Option<i32>>(0))?;
    let mut values = Vec::new();
    for row in rows {
        if let Some(value) = row? {
            values.push(f64::from(value));
        }
    }
    if values.len() < MIN_PERF_SAMPLE as usize {
        return Ok((None, None));
    }
    let len = values.len() as f64;
    let mean = values.iter().sum::<f64>() / len;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / len;
    Ok((Some(mean), Some(variance.sqrt())))
}

fn color_record(conn: &Connection, color: &str) -> AppResult<ColorRecord> {
    let (win_result, loss_result, accuracy_column) = if color == "white" {
        ("1-0", "0-1", "white_accuracy")
    } else {
        ("0-1", "1-0", "black_accuracy")
    };
    let sql = format!(
        "SELECT COUNT(*),
            COALESCE(SUM(CASE WHEN g.result = '{win_result}' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result = '{loss_result}' THEN 1 ELSE 0 END), 0),
            AVG(a.{accuracy_column})
         FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id
         WHERE g.user_color = '{color}'"
    );
    let row = conn.query_row(&sql, [], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, Option<f64>>(4)?,
        ))
    })?;
    Ok(ColorRecord {
        games: row.0 as u32,
        wins: row.1 as u32,
        draws: row.2 as u32,
        losses: row.3 as u32,
        accuracy: row.4,
    })
}

/// Минимальная выборка, при которой точность дебюта считается показательной.
const MIN_RELIABLE_SAMPLE: u32 = 5;

fn openings(conn: &Connection) -> AppResult<Vec<OpeningStat>> {
    let mut stmt = conn.prepare(
        "SELECT g.eco, COALESCE(g.opening_name, ''), COUNT(*),
            COALESCE(SUM(CASE WHEN a.id IS NOT NULL THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='1-0') OR (g.user_color='black' AND g.result='0-1') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result='1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='0-1') OR (g.user_color='black' AND g.result='1-0') THEN 1 ELSE 0 END), 0),
            AVG(CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END)
         FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id
         WHERE g.user_color IN ('white', 'black') AND g.eco IS NOT NULL AND g.eco <> ''
         GROUP BY g.eco, COALESCE(g.opening_name, '')
         ORDER BY COUNT(*) DESC, g.eco ASC
         LIMIT 40",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, Option<f64>>(7)?,
        ))
    })?;
    let mut stats = Vec::new();
    for row in rows {
        let (eco, name, games, analyzed, wins, draws, losses, accuracy) = row?;
        let (exit_win_prob, exit_eval, avg_loss) = exit_metrics(conn, &eco, &name)?;
        let analyzed = analyzed as u32;
        stats.push(OpeningStat {
            eco,
            name,
            games: games as u32,
            analyzed,
            wins: wins as u32,
            draws: draws as u32,
            losses: losses as u32,
            win_rate: if games > 0 {
                f64::from(wins as u32) / f64::from(games as u32) * 100.0
            } else {
                0.0
            },
            accuracy,
            exit_eval,
            exit_win_prob,
            price: f64::from(games as u32) * avg_loss.unwrap_or(0.0),
            reliable: analyzed >= MIN_RELIABLE_SAMPLE,
        });
    }
    Ok(stats)
}

fn exit_metrics(
    conn: &Connection,
    eco: &str,
    name: &str,
) -> AppResult<(Option<f64>, Option<f64>, Option<f64>)> {
    conn.query_row(
        "SELECT
            AVG(CASE WHEN g.user_color='white' THEN me.win_prob_white ELSE 100.0 - me.win_prob_white END),
            AVG(CASE WHEN g.user_color='white' THEN me.eval_cp ELSE -me.eval_cp END),
            (SELECT AVG(me2.win_prob_loss)
             FROM move_evaluations me2 JOIN games g2 ON g2.id = me2.game_id
             WHERE g2.eco = ?1 AND COALESCE(g2.opening_name, '') = ?2
               AND ((g2.user_color='white' AND me2.ply % 2 = 1)
                    OR (g2.user_color='black' AND me2.ply % 2 = 0)))
         FROM move_evaluations me JOIN games g ON g.id = me.game_id
         WHERE g.eco = ?1 AND COALESCE(g.opening_name, '') = ?2 AND me.phase = 'opening'
           AND me.ply = (
                SELECT MAX(m2.ply) FROM move_evaluations m2
                WHERE m2.game_id = me.game_id AND m2.phase = 'opening'
           )",
        params![eco, name],
        |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?,
                row.get::<_, Option<f64>>(1)?.map(|value| value / 100.0),
                row.get::<_, Option<f64>>(2)?,
            ))
        },
    )
    .map_err(Into::into)
}

fn phases(conn: &Connection) -> AppResult<Vec<PhaseStat>> {
    let row = conn.query_row(
        "SELECT
            AVG(CASE WHEN g.user_color='white' THEN a.white_opening_acc ELSE a.black_opening_acc END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_middlegame_acc ELSE a.black_middlegame_acc END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_endgame_acc ELSE a.black_endgame_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_opening_acc ELSE a.black_opening_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_middlegame_acc ELSE a.black_middlegame_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_endgame_acc ELSE a.black_endgame_acc END)
         FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE g.user_color IN ('white', 'black')",
        [],
        |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?,
                row.get::<_, Option<f64>>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        },
    )?;
    let counts = phase_move_counts(conn)?;
    Ok(vec![
        PhaseStat {
            phase: Phase::Opening,
            accuracy: row.0,
            moves: counts.0,
            games: row.3 as u32,
        },
        PhaseStat {
            phase: Phase::Middlegame,
            accuracy: row.1,
            moves: counts.1,
            games: row.4 as u32,
        },
        PhaseStat {
            phase: Phase::Endgame,
            accuracy: row.2,
            moves: counts.2,
            games: row.5 as u32,
        },
    ])
}

fn phase_move_counts(conn: &Connection) -> AppResult<(u32, u32, u32)> {
    let mut stmt = conn.prepare(
        "SELECT me.phase, COUNT(*)
         FROM move_evaluations me JOIN games g ON g.id = me.game_id
         WHERE (g.user_color='white' AND me.ply % 2 = 1) OR (g.user_color='black' AND me.ply % 2 = 0)
         GROUP BY me.phase",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut opening = 0;
    let mut middlegame = 0;
    let mut endgame = 0;
    for row in rows {
        let (phase, count) = row?;
        match phase.as_str() {
            "opening" => opening = count as u32,
            "middlegame" => middlegame = count as u32,
            "endgame" => endgame = count as u32,
            _ => {}
        }
    }
    Ok((opening, middlegame, endgame))
}
