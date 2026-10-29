use rusqlite::Row;

use crate::domain::game::{Game, GameListItem, GameSource, TimeClass, UserColor};

pub(super) const GAME_COLUMNS: &str = "g.id, g.external_id, g.source, g.pgn, g.initial_fen, g.white_username, \
    g.black_username, g.white_elo, g.black_elo, g.user_color, g.result, g.termination, \
    g.time_control, g.time_class, g.eco, g.opening_name, g.opening_ply, g.opening_source, \
    g.played_at, g.total_plies, g.notes, g.is_favorite, g.analysis_attempts, g.last_analysis_error";

pub(super) const LIST_COLUMNS: &str = "g.id, g.white_username, g.black_username, g.white_elo, g.black_elo, \
    g.user_color, g.result, g.time_control, g.time_class, g.eco, g.opening_name, g.played_at, \
    g.total_plies, g.notes, g.is_favorite, g.analysis_attempts, g.last_analysis_error";

pub(super) fn map_game(row: &Row<'_>) -> rusqlite::Result<Game> {
    let source = row.get::<_, String>("source")?;
    let user_color = row.get::<_, String>("user_color")?;
    let time_class = row.get::<_, Option<String>>("time_class")?;
    Ok(Game {
        id: row.get("id")?,
        external_id: row.get("external_id")?,
        source: GameSource::from_str_opt(&source).unwrap_or(GameSource::Manual),
        pgn: row.get("pgn")?,
        initial_fen: row.get("initial_fen")?,
        white_username: row.get("white_username")?,
        black_username: row.get("black_username")?,
        white_elo: row.get("white_elo")?,
        black_elo: row.get("black_elo")?,
        user_color: UserColor::from_str_opt(&user_color).unwrap_or(UserColor::None),
        result: row.get("result")?,
        termination: row.get("termination")?,
        time_control: row.get("time_control")?,
        time_class: time_class.and_then(|value| TimeClass::from_str_opt(&value)),
        eco: row.get("eco")?,
        opening_name: row.get("opening_name")?,
        opening_ply: row.get("opening_ply")?,
        opening_source: row.get("opening_source")?,
        played_at: row.get("played_at")?,
        total_plies: row.get("total_plies")?,
        notes: row.get("notes")?,
        is_favorite: row.get::<_, i64>("is_favorite")? != 0,
        analysis_attempts: row.get::<_, i64>("analysis_attempts")? as u32,
        last_analysis_error: row.get("last_analysis_error")?,
    })
}

pub(super) fn map_game_list_item(row: &Row<'_>) -> rusqlite::Result<GameListItem> {
    let user_color = row.get::<_, String>("user_color")?;
    let user_color = UserColor::from_str_opt(&user_color).unwrap_or(UserColor::None);
    let white_accuracy = row.get::<_, Option<f64>>("white_accuracy")?;
    let black_accuracy = row.get::<_, Option<f64>>("black_accuracy")?;
    let white_acpl = row.get::<_, Option<f64>>("white_acpl")?;
    let black_acpl = row.get::<_, Option<f64>>("black_acpl")?;
    let time_class = row.get::<_, Option<String>>("time_class")?;
    Ok(GameListItem {
        id: row.get("id")?,
        white_username: row.get("white_username")?,
        black_username: row.get("black_username")?,
        white_elo: row.get("white_elo")?,
        black_elo: row.get("black_elo")?,
        user_color,
        result: row.get("result")?,
        time_class: time_class.and_then(|value| TimeClass::from_str_opt(&value)),
        time_control: row.get("time_control")?,
        eco: row.get("eco")?,
        opening_name: row.get("opening_name")?,
        played_at: row.get("played_at")?,
        total_plies: row.get("total_plies")?,
        is_favorite: row.get::<_, i64>("is_favorite")? != 0,
        notes: row.get("notes")?,
        has_analysis: row.get::<_, i64>("has_analysis")? != 0,
        user_accuracy: match user_color {
            UserColor::White => white_accuracy,
            UserColor::Black => black_accuracy,
            UserColor::None => None,
        },
        user_acpl: match user_color {
            UserColor::White => white_acpl,
            UserColor::Black => black_acpl,
            UserColor::None => None,
        },
        key_moments_count: row.get("key_moments_count")?,
        engine_name: row.get("engine_name")?,
        analysis_depth: row.get("depth")?,
        accuracy_method: row.get("accuracy_method_version")?,
        analysis_attempts: row.get::<_, i64>("analysis_attempts")? as u32,
        last_analysis_error: row.get("last_analysis_error")?,
    })
}
