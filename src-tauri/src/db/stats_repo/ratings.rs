use rusqlite::{params, Connection};

use crate::domain::error::AppResult;
use crate::domain::game::GameSource;
use crate::domain::stats::PlatformRating;

/// Performance Elo показываем только при достаточной выборке внутри одной
/// площадки/аккаунта, чтобы не смешивать разные рейтинговые системы.
const MIN_PERF_SAMPLE: u32 = 20;

pub(super) fn platform_ratings(
    conn: &Connection,
    key: Option<&str>,
    account: Option<i64>,
) -> AppResult<Vec<PlatformRating>> {
    let account_filter = super::account_clause("g", 1);
    let own_elo = "CASE WHEN g.user_color='white' THEN g.white_elo ELSE g.black_elo END";
    let opponent_elo = "CASE WHEN g.user_color='white' THEN g.black_elo ELSE g.white_elo END";
    let perf_elo = "CASE WHEN g.user_color='white' THEN a.white_perf_elo ELSE a.black_perf_elo END";
    let sql = format!(
        "SELECT
            g.account_id,
            COALESCE(acc.platform, g.source) AS platform,
            acc.username AS account_username,
            COUNT(*) AS games,
            COALESCE(SUM(CASE WHEN a.id IS NOT NULL THEN 1 ELSE 0 END), 0) AS analyzed,
            AVG({own_elo}) AS avg_user_elo,
            AVG({opponent_elo}) AS avg_opponent_elo,
            AVG({perf_elo}) AS avg_perf,
            COUNT({perf_elo}) AS perf_n,
            AVG(({perf_elo}) * ({perf_elo})) AS avg_perf_sq
         FROM games g
         LEFT JOIN accounts acc ON acc.id = g.account_id
         LEFT JOIN game_analyses a
           ON a.game_id = g.id AND (?2 IS NULL OR a.config_key = ?2)
         WHERE g.user_color IN ('white', 'black') AND {account_filter}
         GROUP BY g.account_id, COALESCE(acc.platform, g.source), acc.username
         ORDER BY platform ASC, account_username COLLATE NOCASE ASC, games DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![account, key], |row| {
        Ok((
            row.get::<_, Option<i64>>("account_id")?,
            row.get::<_, String>("platform")?,
            row.get::<_, Option<String>>("account_username")?,
            row.get::<_, i64>("games")?,
            row.get::<_, i64>("analyzed")?,
            row.get::<_, Option<f64>>("avg_user_elo")?,
            row.get::<_, Option<f64>>("avg_opponent_elo")?,
            row.get::<_, Option<f64>>("avg_perf")?,
            row.get::<_, i64>("perf_n")?,
            row.get::<_, Option<f64>>("avg_perf_sq")?,
        ))
    })?;
    let mut ratings = Vec::new();
    for row in rows {
        let (
            account_id,
            platform,
            account_username,
            games,
            analyzed,
            avg_user_elo,
            avg_opponent_elo,
            avg_perf,
            perf_n,
            avg_perf_sq,
        ) = row?;
        let perf_games = perf_n.max(0) as u32;
        let (perf_elo, perf_elo_spread) = performance_summary(perf_games, avg_perf, avg_perf_sq);
        ratings.push(PlatformRating {
            account_id,
            account_username,
            platform: GameSource::from_str_opt(&platform).unwrap_or(GameSource::Manual),
            games: games.max(0) as u32,
            analyzed: analyzed.max(0) as u32,
            avg_user_elo,
            avg_opponent_elo,
            perf_elo,
            perf_elo_spread,
            perf_elo_games: perf_games,
        });
    }
    Ok(ratings)
}

fn performance_summary(
    samples: u32,
    mean: Option<f64>,
    mean_square: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    if samples < MIN_PERF_SAMPLE {
        return (None, None);
    }
    let Some(mean) = mean else {
        return (None, None);
    };
    let variance = mean_square
        .map(|square| (square - mean * mean).max(0.0))
        .unwrap_or(0.0);
    (Some(mean), Some(variance.sqrt()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn perf_requires_twenty_samples() {
        assert_eq!(
            performance_summary(19, Some(1500.0), Some(2_250_000.0)),
            (None, None)
        );
        let (mean, spread) = performance_summary(20, Some(1500.0), Some(2_250_000.0));
        assert_eq!(mean, Some(1500.0));
        assert_eq!(spread, Some(0.0));
    }
}
