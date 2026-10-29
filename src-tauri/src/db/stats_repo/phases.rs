use rusqlite::{params, Connection};

use crate::domain::analysis::Phase;
use crate::domain::error::AppResult;
use crate::domain::stats::PhaseStat;

pub(super) fn phases(conn: &Connection, key: Option<&str>) -> AppResult<Vec<PhaseStat>> {
    let row = conn.query_row(
        "SELECT
            AVG(CASE WHEN g.user_color='white' THEN a.white_opening_acc ELSE a.black_opening_acc END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_middlegame_acc ELSE a.black_middlegame_acc END),
            AVG(CASE WHEN g.user_color='white' THEN a.white_endgame_acc ELSE a.black_endgame_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_opening_acc ELSE a.black_opening_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_middlegame_acc ELSE a.black_middlegame_acc END),
            COUNT(CASE WHEN g.user_color='white' THEN a.white_endgame_acc ELSE a.black_endgame_acc END)
         FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE g.user_color IN ('white', 'black') AND (?1 IS NULL OR a.config_key = ?1)",
        params![key],
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
    let counts = phase_move_counts(conn, key)?;
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

fn phase_move_counts(conn: &Connection, key: Option<&str>) -> AppResult<(u32, u32, u32)> {
    let mut stmt = conn.prepare(
        "SELECT me.phase, COUNT(*)
         FROM move_evaluations me
         JOIN games g ON g.id = me.game_id
         JOIN game_analyses a ON a.game_id = g.id AND (?1 IS NULL OR a.config_key = ?1)
         WHERE (g.user_color='white' AND me.ply % 2 = 1) OR (g.user_color='black' AND me.ply % 2 = 0)
         GROUP BY me.phase",
    )?;
    let rows = stmt.query_map(params![key], |row| {
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
