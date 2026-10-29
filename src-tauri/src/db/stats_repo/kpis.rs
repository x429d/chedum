use rusqlite::{params, Connection};

use crate::domain::error::AppResult;
use crate::domain::stats::{ColorRecord, HubKpis};

const MIN_PERF_SAMPLE: u32 = 20;

pub(super) fn kpis(conn: &Connection, key: Option<&str>) -> AppResult<HubKpis> {
    let row = conn.query_row(
        "SELECT
            COUNT(*),
            COALESCE(SUM(CASE WHEN a.id IS NOT NULL THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='1-0') OR (g.user_color='black' AND g.result='0-1') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result='1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='0-1') OR (g.user_color='black' AND g.result='1-0') THEN 1 ELSE 0 END), 0),
            AVG(CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_acpl ELSE a.black_acpl END)
         FROM games g LEFT JOIN game_analyses a
              ON a.game_id = g.id AND (?1 IS NULL OR a.config_key = ?1)
         WHERE g.user_color IN ('white', 'black')",
        params![key],
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
    let (avg_perf_elo, avg_perf_elo_spread) = performance_summary(conn, analyzed, key)?;
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
        white: color_record(conn, "white", key)?,
        black: color_record(conn, "black", key)?,
    })
}

fn performance_summary(
    conn: &Connection,
    analyzed: u32,
    key: Option<&str>,
) -> AppResult<(Option<f64>, Option<f64>)> {
    if analyzed < MIN_PERF_SAMPLE {
        return Ok((None, None));
    }
    let mut stmt = conn.prepare(
        "SELECT CASE WHEN g.user_color='white' THEN a.white_perf_elo ELSE a.black_perf_elo END
         FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE g.user_color IN ('white', 'black') AND (?1 IS NULL OR a.config_key = ?1)",
    )?;
    let rows = stmt.query_map(params![key], |row| row.get::<_, Option<i32>>(0))?;
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

fn color_record(conn: &Connection, color: &str, key: Option<&str>) -> AppResult<ColorRecord> {
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
         FROM games g LEFT JOIN game_analyses a
              ON a.game_id = g.id AND (?1 IS NULL OR a.config_key = ?1)
         WHERE g.user_color = '{color}'"
    );
    let row = conn.query_row(&sql, params![key], |row| {
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
