use rusqlite::OptionalExtension;

use crate::db::connection::Db;
use crate::domain::analysis::{
    ACCURACY_METHOD_VERSION, CLASSIFIER_VERSION, DEFAULT_ENGINE_VERSION,
};
use crate::domain::error::AppResult;
use crate::domain::stats::AppSettings;

/// Configuration key of the next analysis: the engine comes from the last saved
/// analysis (Stockfish 19 before the first one), depth and MultiPV from the
/// current settings.
///
/// Rows written by older builds may have NULL in `engine_version`: skip such
/// rows, otherwise reading the column as String fails the entire queue query.
pub fn target_config_key(db: &Db, settings: &AppSettings) -> AppResult<String> {
    let (engine, version) = db.with(|conn| {
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT engine_name, engine_version FROM game_analyses
                 WHERE engine_name IS NOT NULL AND engine_name <> ''
                   AND engine_version IS NOT NULL AND engine_version <> ''
                 ORDER BY analyzed_at DESC, id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(row.unwrap_or_else(|| {
            (
                format!("Stockfish {DEFAULT_ENGINE_VERSION}"),
                DEFAULT_ENGINE_VERSION.to_string(),
            )
        }))
    })?;
    Ok(format!(
        "{}|{}|{}|{}|{}|{}",
        engine,
        version,
        settings.analysis_depth,
        settings.multipv,
        CLASSIFIER_VERSION,
        ACCURACY_METHOD_VERSION
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::stats::AppSettings;

    #[test]
    fn default_target_is_the_bundled_engine_and_settings() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings::default();
        let key = target_config_key(&db, &settings).unwrap();
        assert_eq!(key, "Stockfish 19|19|14|2|classifier-v1|lichess-v1");
    }

    #[test]
    fn depth_change_changes_the_target() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings {
            analysis_depth: 18,
            ..AppSettings::default()
        };
        let key = target_config_key(&db, &settings).unwrap();
        assert!(key.contains("|18|"), "{key}");
    }
}
