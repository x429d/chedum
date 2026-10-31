use rusqlite::params_from_iter;
use rusqlite::types::Value;

use crate::db::connection::Db;
use crate::db::games_repo::mapping::{map_game_list_item, LIST_COLUMNS};
use crate::domain::error::AppResult;
use crate::domain::game::{GameFilter, PagedGames};

pub(super) fn filter_clause(filter: &GameFilter) -> (String, Vec<Value>) {
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
    if let Some(account_id) = filter.account_id {
        clauses.push("g.account_id = ?".to_string());
        values.push(Value::Integer(account_id));
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
            "FROM games g LEFT JOIN game_analyses a ON a.game_id = g.id
             LEFT JOIN accounts acc ON acc.id = g.account_id {where_sql}"
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
             a.white_acpl, a.black_acpl, a.key_moments_count, a.engine_name, a.depth, \
             a.accuracy_method_version {base} ORDER BY {order} LIMIT {limit} OFFSET {offset}"
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
