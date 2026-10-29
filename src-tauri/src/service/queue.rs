use rusqlite::OptionalExtension;

use crate::db::connection::Db;
use crate::domain::analysis::{ACCURACY_METHOD_VERSION, CLASSIFIER_VERSION};
use crate::domain::error::AppResult;
use crate::domain::stats::AppSettings;

/// Ключ конфигурации, которую выдаст следующий анализ: движок берётся из
/// последнего сохранённого анализа (до первого — Stockfish 19), глубина и
/// MultiPV — из текущих настроек.
pub fn target_config_key(db: &Db, settings: &AppSettings) -> AppResult<String> {
    let (engine, version) = db.with(|conn| {
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT engine_name, engine_version FROM game_analyses
                 ORDER BY analyzed_at DESC, id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(row.unwrap_or_else(|| ("Stockfish 19".to_string(), "19".to_string())))
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
