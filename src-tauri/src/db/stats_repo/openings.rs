use rusqlite::{params, Connection};

use crate::domain::error::AppResult;
use crate::domain::stats::OpeningStat;

const MIN_RELIABLE_SAMPLE: u32 = 5;

pub(super) fn openings(
    conn: &Connection,
    key: Option<&str>,
    account: Option<&str>,
) -> AppResult<Vec<OpeningStat>> {
    let account_filter = super::account_clause("g", 1);
    let sql = format!(
        "SELECT g.eco, COALESCE(g.opening_name, ''), COUNT(*),
            COALESCE(SUM(CASE WHEN a.id IS NOT NULL THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='1-0') OR (g.user_color='black' AND g.result='0-1') THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN g.result='1/2-1/2' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN (g.user_color='white' AND g.result='0-1') OR (g.user_color='black' AND g.result='1-0') THEN 1 ELSE 0 END), 0),
            AVG(CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END)
         FROM games g LEFT JOIN game_analyses a
              ON a.game_id = g.id AND (?2 IS NULL OR a.config_key = ?2)
         WHERE g.user_color IN ('white', 'black') AND g.eco IS NOT NULL AND g.eco <> ''
           AND {account_filter}
         GROUP BY g.eco, COALESCE(g.opening_name, '')
         ORDER BY COUNT(*) DESC, g.eco ASC
         LIMIT 40"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![account, key], |row| {
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
        let (exit_win_prob, exit_eval, avg_loss) = exit_metrics(conn, &eco, &name, key, account)?;
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
    key: Option<&str>,
    account: Option<&str>,
) -> AppResult<(Option<f64>, Option<f64>, Option<f64>)> {
    let account_filter = super::account_clause("g2", 4);
    let outer_account_filter = super::account_clause("g", 4);
    let sql = format!(
        "SELECT
            AVG(CASE WHEN g.user_color='white' THEN me.win_prob_white ELSE 100.0 - me.win_prob_white END),
            AVG(CASE WHEN g.user_color='white' THEN me.eval_cp ELSE -me.eval_cp END),
            (SELECT AVG(me2.win_prob_loss)
             FROM move_evaluations me2
             JOIN games g2 ON g2.id = me2.game_id
             JOIN game_analyses a2 ON a2.game_id = g2.id AND (?3 IS NULL OR a2.config_key = ?3)
             WHERE g2.eco = ?1 AND COALESCE(g2.opening_name, '') = ?2
               AND me2.player_color = g2.user_color
               AND {account_filter})
         FROM move_evaluations me
         JOIN games g ON g.id = me.game_id
         JOIN game_analyses a ON a.game_id = g.id AND (?3 IS NULL OR a.config_key = ?3)
         WHERE g.eco = ?1 AND COALESCE(g.opening_name, '') = ?2 AND me.phase = 'opening'
           AND {outer_account_filter}
           AND me.ply = (
                SELECT MAX(m2.ply) FROM move_evaluations m2
                WHERE m2.game_id = me.game_id AND m2.phase = 'opening'
           )"
    );
    conn.query_row(&sql, params![eco, name, key, account], |row| {
        Ok((
            row.get::<_, Option<f64>>(0)?,
            row.get::<_, Option<f64>>(1)?.map(|value| value / 100.0),
            row.get::<_, Option<f64>>(2)?,
        ))
    })
    .map_err(Into::into)
}
