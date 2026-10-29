use crate::db::connection::Db;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{
    Game, GameDraft, GameFilter, GameListItem, GameSource, PagedGames, PendingAnalysis, TimeClass,
    UserColor,
};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

const GAME_COLUMNS: &str = "g.id, g.external_id, g.source, g.pgn, g.initial_fen, g.white_username, \
    g.black_username, g.white_elo, g.black_elo, g.user_color, g.result, g.termination, \
    g.time_control, g.time_class, g.eco, g.opening_name, g.played_at, g.total_plies, g.notes, g.is_favorite";

const LIST_COLUMNS: &str = "g.id, g.white_username, g.black_username, g.white_elo, g.black_elo, \
    g.user_color, g.result, g.time_control, g.time_class, g.eco, g.opening_name, g.played_at, \
    g.total_plies, g.notes, g.is_favorite";

pub fn insert_game(conn: &Connection, draft: &GameDraft) -> AppResult<Option<i64>> {
    let changed = conn.execute(
        "INSERT OR IGNORE INTO games (
            external_id, source, pgn, initial_fen, white_username, black_username, white_elo,
            black_elo, user_color, result, termination, time_control, time_class, eco,
            opening_name, played_at, total_plies, notes, is_favorite
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            draft.external_id,
            draft.source.as_str(),
            draft.pgn,
            draft.initial_fen,
            draft.white_username,
            draft.black_username,
            draft.white_elo,
            draft.black_elo,
            draft.user_color.as_str(),
            draft.result,
            draft.termination,
            draft.time_control,
            draft.time_class.map(TimeClass::as_str),
            draft.eco,
            draft.opening_name,
            draft.played_at,
            draft.total_plies,
            draft.notes,
            i32::from(draft.is_favorite),
        ],
    )?;
    if changed == 0 {
        Ok(None)
    } else {
        Ok(Some(conn.last_insert_rowid()))
    }
}

pub fn count_games(db: &Db) -> AppResult<u32> {
    db.with(|conn| {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))?;
        Ok(count as u32)
    })
}

pub fn get_game(db: &Db, id: i64) -> AppResult<Game> {
    db.with(|conn| {
        conn.query_row(
            &format!("SELECT {GAME_COLUMNS} FROM games g WHERE g.id = ?1"),
            params![id],
            map_game,
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("Партия #{id} не найдена")))
    })
}

pub fn set_favorite(db: &Db, id: i64, favorite: bool) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET is_favorite = ?1 WHERE id = ?2",
            params![i32::from(favorite), id],
        )?;
        Ok(())
    })
}

pub fn update_notes(db: &Db, id: i64, notes: &str) -> AppResult<()> {
    db.with(|conn| {
        conn.execute(
            "UPDATE games SET notes = ?1 WHERE id = ?2",
            params![notes, id],
        )?;
        Ok(())
    })
}

pub fn games_by_ids(db: &Db, ids: &[i64]) -> AppResult<Vec<Game>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    db.with(|conn| {
        let placeholders = vec!["?"; ids.len()].join(",");
        let sql = format!(
            "SELECT {GAME_COLUMNS} FROM games g WHERE g.id IN ({placeholders}) ORDER BY g.played_at ASC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(ids.iter()), map_game)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(games)
    })
}

pub fn all_games(db: &Db) -> AppResult<Vec<Game>> {
    db.with(|conn| {
        let sql = format!("SELECT {GAME_COLUMNS} FROM games g ORDER BY g.played_at ASC");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], map_game)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(games)
    })
}

/// Собирает WHERE-условия выборки и параметры для них.
fn filter_clause(filter: &GameFilter) -> (String, Vec<Value>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    if let Some(search) = filter
        .search
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        let like = format!("%{}%", search.trim());
        clauses.push(
            "(g.white_username LIKE ? OR g.black_username LIKE ? OR g.opening_name LIKE ? OR g.eco LIKE ?)"
                .to_string(),
        );
        for _ in 0..4 {
            values.push(Value::Text(like.clone()));
        }
    }
    if let Some(color) = filter.color {
        clauses.push("g.user_color = ?".to_string());
        values.push(Value::Text(color.as_str().to_string()));
    }
    match filter.result.as_deref() {
        Some("win") => clauses.push(
            "((g.user_color = 'white' AND g.result = '1-0') OR (g.user_color = 'black' AND g.result = '0-1'))"
                .to_string(),
        ),
        Some("loss") => clauses.push(
            "((g.user_color = 'white' AND g.result = '0-1') OR (g.user_color = 'black' AND g.result = '1-0'))"
                .to_string(),
        ),
        Some("draw") => clauses.push("g.result = '1/2-1/2'".to_string()),
        _ => {}
    }
    if let Some(time_class) = filter.time_class {
        clauses.push("g.time_class = ?".to_string());
        values.push(Value::Text(time_class.as_str().to_string()));
    }
    if let Some(eco) = filter.eco.as_ref().filter(|value| !value.trim().is_empty()) {
        clauses.push("g.eco LIKE ?".to_string());
        values.push(Value::Text(format!("{}%", eco.trim().to_uppercase())));
    }
    if let Some(from) = filter.date_from {
        clauses.push("g.played_at >= ?".to_string());
        values.push(Value::Integer(from));
    }
    if let Some(to) = filter.date_to {
        clauses.push("g.played_at <= ?".to_string());
        values.push(Value::Integer(to));
    }
    if filter.analyzed_only {
        clauses.push("a.id IS NOT NULL".to_string());
    }
    if filter.favorites_only {
        clauses.push("g.is_favorite = 1".to_string());
    }
    if filter.has_notes {
        clauses.push("g.notes <> ''".to_string());
    }
    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    (where_sql, values)
}

pub fn list_games(db: &Db, filter: &GameFilter) -> AppResult<PagedGames> {
    db.with(|conn| {
        let (where_sql, values) = filter_clause(filter);
        let base = format!(
            "FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id {where_sql}"
        );
        // Счётчики считаются по всей выборке, а не по текущей странице.
        let (total, analyzed, favorites): (i64, i64, i64) = conn.query_row(
            &format!(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN a.id IS NULL THEN 0 ELSE 1 END), 0),
                        COALESCE(SUM(g.is_favorite), 0) {base}"
            ),
            params_from_iter(values.iter()),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let limit = filter.limit.unwrap_or(100).clamp(1, 500);
        let offset = filter.offset.unwrap_or(0);
        let order = match filter.sort.as_deref() {
            Some("date_asc") => "g.played_at ASC",
            Some("accuracy_desc") => {
                "CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END DESC"
            }
            Some("accuracy_asc") => {
                "CASE WHEN g.user_color='white' THEN a.white_accuracy ELSE a.black_accuracy END ASC"
            }
            _ => "g.played_at DESC",
        };
        let sql = format!(
            "SELECT {LIST_COLUMNS}, a.id IS NOT NULL AS has_analysis, a.white_accuracy, a.black_accuracy, \
             a.white_acpl, a.black_acpl, a.key_moments_count {base} ORDER BY {order} LIMIT {limit} OFFSET {offset}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values.iter()), map_game_list_item)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(PagedGames {
            games,
            total: total as u32,
            analyzed: analyzed as u32,
            favorites: favorites as u32,
            limit,
            offset,
        })
    })
}

/// План пакетного анализа: все партии выборки без анализа и число позиций.
pub fn pending_analysis(db: &Db, filter: &GameFilter) -> AppResult<PendingAnalysis> {
    db.with(|conn| {
        let mut filter = filter.clone();
        filter.analyzed_only = false;
        let (where_sql, values) = filter_clause(&filter);
        let condition = if where_sql.is_empty() {
            "WHERE a.id IS NULL".to_string()
        } else {
            format!("{where_sql} AND a.id IS NULL")
        };
        let sql = format!(
            "SELECT g.id, g.total_plies FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id \
             {condition} ORDER BY g.played_at DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?))
        })?;
        let mut ids = Vec::new();
        let mut positions = 0u64;
        for row in rows {
            let (id, plies) = row?;
            ids.push(id);
            positions += u64::from(plies.max(0) as u32) + 1;
        }
        Ok(PendingAnalysis { ids, positions })
    })
}

fn map_game(row: &Row<'_>) -> rusqlite::Result<Game> {
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
        played_at: row.get("played_at")?,
        total_plies: row.get("total_plies")?,
        notes: row.get("notes")?,
        is_favorite: row.get::<_, i64>("is_favorite")? != 0,
    })
}

fn map_game_list_item(row: &Row<'_>) -> rusqlite::Result<GameListItem> {
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
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "repo_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "pagination_tests.rs"]
mod pagination_tests;
