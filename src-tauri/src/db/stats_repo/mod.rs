use rusqlite::Connection;

use crate::db::connection::Db;
use crate::db::stats_extra::{time_stats, trend};
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

pub fn hub_stats(db: &Db) -> AppResult<HubStats> {
    db.with(|conn| {
        let config = select_config(conn)?;
        let key = config.as_ref().map(|value| value.config_key.as_str());
        Ok(HubStats {
            kpis: kpis(conn, key)?,
            openings: openings(conn, key)?,
            phases: phases(conn, key)?,
            trend: trend(conn, key)?,
            time: time_stats(conn, key)?,
            config: config.map(|value| value.view),
        })
    })
}

/// Самая частая конфигурация среди разобранных партий. Остальные анализы
/// не удаляются, но в агрегаты хаба не попадают.
fn select_config(conn: &Connection) -> AppResult<Option<ConfigSelection>> {
    let row = conn.query_row(
        "SELECT engine_name, engine_version, depth, multipv, classifier_version,
                accuracy_method_version, config_key, COUNT(*)
         FROM game_analyses
         GROUP BY config_key
         ORDER BY COUNT(*) DESC, depth DESC, analyzed_at DESC
         LIMIT 1",
        [],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
            ))
        },
    );
    let (engine_name, engine_version, depth, multipv, classifier, method, config_key, included) =
        match row {
            Ok(value) => value,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM game_analyses", [], |row| row.get(0))?;
    let mut stmt = conn.prepare(
        "SELECT game_id FROM game_analyses WHERE config_key <> ?1 ORDER BY game_id",
    )?;
    let rows = stmt.query_map([&config_key], |row| row.get::<_, i64>(0))?;
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
