use rusqlite::{params, Connection};

use crate::db::connection::Db;
use crate::db::stats_extra::{time_stats, trend};
use crate::domain::analysis::DEFAULT_ENGINE_VERSION;
use crate::domain::error::AppResult;
use crate::domain::stats::{AnalysisConfigView, HubStats};

mod kpis;
mod openings;
mod phases;

use kpis::kpis;
use openings::openings;
use phases::phases;

struct ConfigSelection {
    config_key: String,
    view: AnalysisConfigView,
}

/// Партии принадлежат аккаунту напрямую через `games.account_id`.
/// `?param = NULL` означает «все аккаунты и ручные PGN».
pub(crate) fn account_clause(alias: &str, param: usize) -> String {
    format!("(?{param} IS NULL OR {alias}.account_id = ?{param})")
}

/// Агрегаты хаба для выбранного аккаунта или для всех аккаунтов.
pub fn hub_stats(db: &Db, account: Option<i64>) -> AppResult<HubStats> {
    hub_stats_for(db, account)
}

fn hub_stats_for(db: &Db, account: Option<i64>) -> AppResult<HubStats> {
    db.with(|conn| {
        let config = select_config(conn, account)?;
        let key = config.as_ref().map(|value| value.config_key.as_str());
        Ok(HubStats {
            kpis: kpis(conn, key, account)?,
            openings: openings(conn, key, account)?,
            phases: phases(conn, key, account)?,
            trend: trend(conn, key, account)?,
            time: time_stats(conn, key, account)?,
            config: config.map(|value| value.view),
        })
    })
}

/// Самая частая конфигурация среди разобранных партий аккаунта. Остальные
/// анализы не удаляются, но в агрегаты хаба не попадают.
fn select_config(conn: &Connection, account: Option<i64>) -> AppResult<Option<ConfigSelection>> {
    let account_filter = account_clause("g", 1);
    let sql = format!(
        "SELECT a.engine_name, a.engine_version, a.depth, a.multipv, a.classifier_version,
                a.accuracy_method_version, a.config_key, COUNT(*)
         FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE a.config_key <> '' AND {account_filter}
         GROUP BY a.config_key
         ORDER BY COUNT(*) DESC, a.depth DESC, a.analyzed_at DESC
         LIMIT 1"
    );
    let row = conn.query_row(&sql, params![account], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?
                .unwrap_or_else(|| DEFAULT_ENGINE_VERSION.to_string()),
            row.get::<_, i32>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?,
        ))
    });
    let (engine_name, engine_version, depth, multipv, classifier, method, config_key, included) =
        match row {
            Ok(value) => value,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
    let total_sql = format!(
        "SELECT COUNT(*) FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE {account_filter}"
    );
    let total: i64 = conn.query_row(&total_sql, params![account], |row| row.get(0))?;
    let mismatch_sql = format!(
        "SELECT a.game_id FROM game_analyses a JOIN games g ON g.id = a.game_id
         WHERE a.config_key <> ?2 AND {account_filter}
         ORDER BY a.game_id"
    );
    let mut stmt = conn.prepare(&mismatch_sql)?;
    let rows = stmt.query_map(params![account, &config_key], |row| row.get::<_, i64>(0))?;
    let mut mismatched = Vec::new();
    for row in rows {
        mismatched.push(row?);
    }
    Ok(Some(ConfigSelection {
        config_key,
        view: AnalysisConfigView {
            engine_name,
            engine_version,
            depth,
            multipv,
            classifier_version: classifier,
            accuracy_method_version: method,
            included: included as u32,
            total: total as u32,
            mismatched_game_ids: mismatched,
        },
    }))
}
