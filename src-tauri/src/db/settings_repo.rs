use std::collections::HashMap;

use rusqlite::{params, Connection};

use crate::db::connection::Db;
use crate::domain::error::{AppError, AppResult};
use crate::domain::stats::{AppSettings, PIECE_SETS};

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
        "analysis_depth",
        "multipv",
        "board_theme",
        "piece_set",
        "show_arrows",
        "auto_analyze",
    ];
    if !allowed.contains(&key) {
        return Err(AppError::Validation(format!(
            "Неизвестный параметр «{key}»"
        )));
    }
    if key == "analysis_depth" {
        let depth: u32 = value
            .parse()
            .map_err(|_| AppError::Validation("Глубина должна быть числом".to_string()))?;
        if !(6..=24).contains(&depth) {
            return Err(AppError::Validation(
                "Глубина анализа должна быть от 6 до 24".to_string(),
            ));
        }
    }
    if key == "multipv" {
        let multipv: u32 = value
            .parse()
            .map_err(|_| AppError::Validation("MultiPV должен быть числом".to_string()))?;
        if !(1..=3).contains(&multipv) {
            return Err(AppError::Validation(
                "MultiPV должен быть от 1 до 3".to_string(),
            ));
        }
    }
    if key == "piece_set" && !PIECE_SETS.contains(&value) {
        return Err(AppError::Validation(format!(
            "Неизвестный набор фигур «{value}»"
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
    db.with_tx(|tx| {
        let values = [
            ("player_username", settings.player_username.clone()),
            ("analysis_depth", settings.analysis_depth.to_string()),
            ("multipv", settings.multipv.to_string()),
            ("board_theme", settings.board_theme.clone()),
            ("piece_set", settings.piece_set.clone()),
            ("show_arrows", settings.show_arrows.to_string()),
            ("auto_analyze", settings.auto_analyze.to_string()),
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
    if let Some(value) = map
        .get("analysis_depth")
        .and_then(|value| value.parse().ok())
    {
        settings.analysis_depth = value;
    }
    if let Some(value) = map.get("multipv").and_then(|value| value.parse().ok()) {
        settings.multipv = value;
    }
    if let Some(value) = map.get("board_theme") {
        settings.board_theme = value.clone();
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
}
