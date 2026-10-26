use rusqlite::Connection;

use crate::domain::error::AppResult;
use crate::domain::game::Color;
use crate::domain::stats::{AccuracyPoint, TimeStats};

pub(crate) fn trend(conn: &Connection) -> AppResult<Vec<AccuracyPoint>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.played_at, g.white_username, g.black_username, g.user_color, g.result,
            CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END
         FROM games g JOIN game_analyses a ON a.game_id = g.id
         WHERE g.user_color IN ('white', 'black')
         ORDER BY g.played_at ASC
         LIMIT 200",
    )?;
    let rows = stmt.query_map([], |row| {
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

pub(crate) fn time_stats(conn: &Connection) -> AppResult<TimeStats> {
    let row = conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN me.time_flag='time_trouble' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN me.time_flag='impulsive_error' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN me.time_flag='overthink' THEN 1 ELSE 0 END), 0),
            AVG(me.time_spent_sec),
            AVG(CASE WHEN me.time_flag IN ('time_trouble', 'impulsive_error') THEN me.move_accuracy END)
         FROM move_evaluations me JOIN games g ON g.id = me.game_id
         WHERE (g.user_color='white' AND me.ply % 2 = 1) OR (g.user_color='black' AND me.ply % 2 = 0)",
        [],
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
