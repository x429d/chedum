use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::db::connection::Db;
use crate::domain::error::{AppError, AppResult};
use crate::domain::stats::{AppSettings, BOARD_THEMES, PIECE_SETS};

pub fn get_settings(db: &Db) -> AppResult<AppSettings> {
    db.with(|conn| {
        let mut stmt = conn.prepare("SELECT key, value FROM app_settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (key, value) = row?;
            map.insert(key, value);
        }
        Ok(settings_from_map(&map))
    })
}

pub fn set_setting(db: &Db, key: &str, value: &str) -> AppResult<()> {
    let allowed = [
        "player_username",
        "active_account_id",
        "analysis_depth",
        "multipv",
        "board_theme",
        "piece_set",
        "show_arrows",
        "auto_analyze",
        "analysis_threads",
        "auto_resume_analysis",
        "language",
    ];
    if !allowed.contains(&key) {
        return Err(AppError::Validation(format!("Unknown setting «{key}»")));
    }
    if key == "language"
        && (!value.chars().all(|c| c.is_ascii_lowercase()) || !(2..=5).contains(&value.len()))
    {
        return Err(AppError::Validation(format!(
            "Invalid language code «{value}»"
        )));
    }
    if key == "active_account_id" && !value.trim().is_empty() {
        value
            .parse::<i64>()
            .map_err(|_| AppError::Validation("Account id must be a number".to_string()))?;
    }
    if key == "analysis_depth" {
        let depth: u32 = value
            .parse()
            .map_err(|_| AppError::Validation("Depth must be a number".to_string()))?;
        if !(6..=24).contains(&depth) {
            return Err(AppError::Validation(
                "Analysis depth must be between 6 and 24".to_string(),
            ));
        }
    }
    if key == "multipv" {
        let multipv: u32 = value
            .parse()
            .map_err(|_| AppError::Validation("MultiPV must be a number".to_string()))?;
        if !(1..=3).contains(&multipv) {
            return Err(AppError::Validation(
                "MultiPV must be between 1 and 3".to_string(),
            ));
        }
    }
    if key == "analysis_threads" {
        let threads: u32 = value
            .parse()
            .map_err(|_| AppError::Validation("Analysis workers must be a number".to_string()))?;
        if threads > 64 {
            return Err(AppError::Validation(
                "Analysis workers must be 0 (auto) or between 1 and 64".to_string(),
            ));
        }
    }
    if key == "piece_set" && !PIECE_SETS.contains(&value) {
        return Err(AppError::Validation(format!("Unknown piece set «{value}»")));
    }
    if key == "board_theme" && !BOARD_THEMES.contains(&value) {
        return Err(AppError::Validation(format!(
            "Unknown board theme «{value}»"
        )));
    }
    db.with(|conn| {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    })
}

pub fn set_many(db: &Db, settings: &AppSettings) -> AppResult<()> {
    settings.validate()?;
    db.with_tx(|tx| {
        let values = [
            ("player_username", settings.player_username.clone()),
            (
                "active_account_id",
                settings
                    .active_account_id
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            ),
            ("analysis_depth", settings.analysis_depth.to_string()),
            ("multipv", settings.multipv.to_string()),
            ("board_theme", settings.board_theme.clone()),
            ("piece_set", settings.piece_set.clone()),
            ("show_arrows", settings.show_arrows.to_string()),
            ("auto_analyze", settings.auto_analyze.to_string()),
            ("analysis_threads", settings.analysis_threads.to_string()),
            (
                "auto_resume_analysis",
                settings.auto_resume_analysis.to_string(),
            ),
            ("language", settings.language.clone()),
        ];
        for (key, value) in values {
            upsert_conn(tx, key, &value)?;
        }
        Ok(())
    })
}

fn upsert_conn(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn settings_from_map(map: &HashMap<String, String>) -> AppSettings {
    let mut settings = AppSettings::default();
    if let Some(value) = map.get("player_username") {
        settings.player_username = value.clone();
    }
    if let Some(value) = map.get("active_account_id") {
        settings.active_account_id = value.trim().parse::<i64>().ok().filter(|id| *id > 0);
    }
    if let Some(value) = map
        .get("analysis_depth")
        .and_then(|value| value.parse().ok())
        .filter(|value| (6..=24).contains(value))
    {
        settings.analysis_depth = value;
    }
    if let Some(value) = map
        .get("multipv")
        .and_then(|value| value.parse().ok())
        .filter(|value| (1..=3).contains(value))
    {
        settings.multipv = value;
    }
    if let Some(value) = map.get("board_theme") {
        if BOARD_THEMES.contains(&value.as_str()) {
            settings.board_theme = value.clone();
        }
    }
    if let Some(value) = map.get("piece_set") {
        if PIECE_SETS.contains(&value.as_str()) {
            settings.piece_set = value.clone();
        }
    }
    if let Some(value) = map.get("show_arrows").and_then(|value| value.parse().ok()) {
        settings.show_arrows = value;
    }
    if let Some(value) = map.get("auto_analyze").and_then(|value| value.parse().ok()) {
        settings.auto_analyze = value;
    }
    if let Some(value) = map
        .get("analysis_threads")
        .and_then(|value| value.parse().ok())
        .filter(|value| *value <= 64)
    {
        settings.analysis_threads = value;
    }
    if let Some(value) = map
        .get("auto_resume_analysis")
        .and_then(|value| value.parse().ok())
    {
        settings.auto_resume_analysis = value;
    }
    if let Some(value) = map.get("language") {
        if value.chars().all(|c| c.is_ascii_lowercase()) && (2..=5).contains(&value.len()) {
            settings.language = value.clone();
        }
    }
    settings
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn piece_set_roundtrips_through_settings() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings {
            piece_set: "celtic".to_string(),
            ..AppSettings::default()
        };
        set_many(&db, &settings).unwrap();
        assert_eq!(get_settings(&db).unwrap().piece_set, "celtic");
    }

    #[test]
    fn new_install_defaults_to_auto_analyze_on() {
        let db = Db::in_memory().unwrap();
        assert!(get_settings(&db).unwrap().auto_analyze);
    }

    #[test]
    fn unknown_piece_set_is_rejected() {
        let db = Db::in_memory().unwrap();
        assert!(set_setting(&db, "piece_set", "unknown").is_err());
    }

    #[test]
    fn unknown_stored_piece_set_falls_back_to_default() {
        let mut map = HashMap::new();
        map.insert("piece_set".to_string(), "unknown".to_string());
        assert_eq!(settings_from_map(&map).piece_set, "cburnett");
    }

    #[test]
    fn language_roundtrips_through_settings() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings {
            language: "ru".to_string(),
            ..AppSettings::default()
        };
        set_many(&db, &settings).unwrap();
        assert_eq!(get_settings(&db).unwrap().language, "ru");
    }

    #[test]
    fn invalid_language_is_rejected() {
        let db = Db::in_memory().unwrap();
        assert!(set_setting(&db, "language", "INVALID_123").is_err());
        assert!(set_setting(&db, "language", "x").is_err());
    }

    #[test]
    fn set_many_rejects_invalid_depth() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings {
            analysis_depth: 99,
            ..AppSettings::default()
        };
        assert!(set_many(&db, &settings).is_err());
        assert_eq!(get_settings(&db).unwrap().analysis_depth, 14);
    }

    #[test]
    fn set_many_rejects_unknown_board_theme() {
        let db = Db::in_memory().unwrap();
        let settings = AppSettings {
            board_theme: "neon".to_string(),
            ..AppSettings::default()
        };
        assert!(set_many(&db, &settings).is_err());
    }
}
