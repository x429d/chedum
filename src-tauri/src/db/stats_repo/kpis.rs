use rusqlite::{params, Connection};

use crate::domain::error::AppResult;
use crate::domain::stats::{ColorRecord, HubKpis};

pub(super) fn kpis(
    conn: &Connection,
    key: Option<&str>,
    account: Option<i64>,
) -> AppResult<HubKpis> {
    let account_filter = super::account_clause("g", 1);
    let sql = format!(
        "SELECT
            COUNT(*),
            COALESCE(SUM(CASE WHEN a.id IS NOT NULL THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='1-0') OR (g.user_color='black' AND g.result='0-1') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result='1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='0-1') OR (g.user_color='black' AND g.result='1-0') THEN 1 ELSE 0 END), 0),
            AVG(CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_acpl ELSE a.black_acpl END)
         FROM games g LEFT JOIN game_analyses a
              ON a.game_id = g.id AND (?2 IS NULL OR a.config_key = ?2)
         WHERE g.user_color IN ('white', 'black') AND {account_filter}"
    );
    let row = conn.query_row(&sql, params![account, key], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Option<f64>>(5)?,
            row.get::<_, Option<f64>>(6)?,
        ))
    })?;
    let total = row.0 as u32;
    let analyzed = row.1 as u32;
    let wins = row.2 as u32;
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
        // Заполняется в `hub_stats` из разбивки по аккаунтам/площадкам.
        avg_perf_elo: None,
        avg_perf_elo_spread: None,
        white: color_record(conn, "white", key, account)?,
        black: color_record(conn, "black", key, account)?,
    })
}

fn color_record(
    conn: &Connection,
    color: &str,
    key: Option<&str>,
    account: Option<i64>,
) -> AppResult<ColorRecord> {
    let (win_result, loss_result, accuracy_column) = if color == "white" {
        ("1-0", "0-1", "white_accuracy")
    } else {
        ("0-1", "1-0", "black_accuracy")
    };
    let account_filter = super::account_clause("g", 1);
    let sql = format!(
        "SELECT COUNT(*),
            COALESCE(SUM(CASE WHEN g.result = '{win_result}' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result = '1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result = '{loss_result}' THEN 1 ELSE 0 END), 0),
            AVG(a.{accuracy_column})
         FROM games g LEFT JOIN game_analyses a
              ON a.game_id = g.id AND (?2 IS NULL OR a.config_key = ?2)
         WHERE g.user_color = '{color}' AND {account_filter}"
    );
    let row = conn.query_row(&sql, params![account, key], |row| {
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
