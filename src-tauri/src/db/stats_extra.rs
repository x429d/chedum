use rusqlite::{params, Connection};

use crate::domain::error::AppResult;
use crate::domain::game::Color;
use crate::domain::stats::{AccuracyPoint, TimeStats};

/// Последние 200 разобранных партий, развёрнутые в хронологический порядок.
/// Пустые даты не попадают в тренд.
pub(crate) fn trend(conn: &Connection, key: Option<&str>) -> AppResult<Vec<AccuracyPoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, played_at, white_username, black_username, user_color, result, accuracy
         FROM (
            SELECT g.id, g.played_at, g.white_username, g.black_username, g.user_color, g.result,
                CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END
                    AS accuracy
            FROM games g
            JOIN game_analyses a ON a.game_id = g.id AND (?1 IS NULL OR a.config_key = ?1)
            WHERE g.user_color IN ('white', 'black') AND g.played_at > 0
            ORDER BY g.played_at DESC, g.id DESC
            LIMIT 200
         )
         ORDER BY played_at ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![key], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<f64>>(6)?,
        ))
    })?;
    let mut points = Vec::new();
    for row in rows {
        let (game_id, played_at, white, black, color, result, accuracy) = row?;
        let Some(accuracy) = accuracy else {
            continue;
        };
        let Some(color) = Color::from_str_opt(&color) else {
            continue;
        };
        let opponent = if color == Color::White { black } else { white };
        let outcome = match (color, result.as_str()) {
            (Color::White, "1-0") | (Color::Black, "0-1") => "win",
            (Color::White, "0-1") | (Color::Black, "1-0") => "loss",
            (_, "1/2-1/2") => "draw",
            _ => "unknown",
        };
        points.push(AccuracyPoint {
            game_id,
            played_at,
            accuracy,
            opponent,
            color,
            result: outcome.to_string(),
        });
    }
    Ok(points)
}

pub(crate) fn time_stats(conn: &Connection, key: Option<&str>) -> AppResult<TimeStats> {
    let row = conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN me.time_flag='time_trouble' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN me.time_flag='impulsive_error' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN me.time_flag='overthink' THEN 1 ELSE 0 END), 0),
            AVG(me.time_spent_sec),
            AVG(CASE WHEN me.time_flag IN ('time_trouble', 'impulsive_error') THEN me.move_accuracy END)
         FROM move_evaluations me
         JOIN games g ON g.id = me.game_id
         JOIN game_analyses a ON a.game_id = g.id AND (?1 IS NULL OR a.config_key = ?1)
         WHERE (g.user_color='white' AND me.ply % 2 = 1) OR (g.user_color='black' AND me.ply % 2 = 0)",
        params![key],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Option<f64>>(4)?,
            ))
        },
    )?;
    Ok(TimeStats {
        time_trouble_moves: row.0 as u32,
        impulsive_errors: row.1 as u32,
        overthink_moves: row.2 as u32,
        avg_time_spent: row.3,
        pressure_accuracy: row.4,
    })
}
