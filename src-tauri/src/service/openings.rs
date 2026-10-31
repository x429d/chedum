use rusqlite::{params, OptionalExtension};

use crate::db::connection::Db;
use crate::domain::error::AppResult;
use crate::eco::{dataset_version, EcoBook};
use crate::pgn::{identify_opening, parse_single};

const OPENINGS_VERSION_KEY: &str = "openings_version";

/// Recalculates openings for all games from their positions. No engine is needed;
/// the operation is idempotent: a second run on the same database does nothing.
pub fn backfill_openings(db: &Db) -> AppResult<u32> {
    let version = dataset_version();
    db.with_tx(|tx| {
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [OPENINGS_VERSION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        if current.as_deref() == Some(version.as_str()) {
            return Ok(0);
        }
        let rows: Vec<(i64, String)> = {
            let mut stmt = tx.prepare("SELECT id, pgn FROM games ORDER BY id")?;
            let mapped = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            mapped.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut updated = 0u32;
        let mut failed = 0u32;
        for (id, pgn) in rows {
            let parsed = match parse_single(&pgn) {
                Ok(parsed) => parsed,
                Err(err) => {
                    failed += 1;
                    tracing::warn!("Opening for game #{id} was not recalculated: {err}");
                    continue;
                }
            };
            let opening = identify_opening(&parsed);
            let changed = tx.execute(
                "UPDATE games SET eco = ?2, opening_name = ?3, opening_ply = ?4,
                    opening_source = ?5
                 WHERE id = ?1 AND (
                    COALESCE(eco, '') <> COALESCE(?2, '')
                    OR COALESCE(opening_name, '') <> COALESCE(?3, '')
                    OR opening_ply IS NOT ?4
                    OR COALESCE(opening_source, '') <> COALESCE(?5, '')
                 )",
                params![id, opening.eco, opening.name, opening.ply, opening.source],
            )?;
            updated += changed as u32;
        }
        tx.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![OPENINGS_VERSION_KEY, version],
        )?;
        if failed > 0 {
            tracing::warn!("Openings were not recalculated for {failed} games");
        }
        Ok(updated)
    })
}

/// Identifies the opening of a single game without writing to the database.
/// Used by the backfill and tests.
pub fn identify_game_pgn(pgn: &str) -> AppResult<Option<(String, String, i32)>> {
    let parsed = parse_single(pgn)?;
    let found = EcoBook::global().identify_with_ply(&parsed.fens_after());
    Ok(found.map(|value| (value.eco, value.name, value.ply as i32)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::db::games_repo::insert_game;
    use crate::domain::game::{GameDraft, GameSource, UserColor};
    use crate::pgn::{build_draft, parse_single};

    const VIENNA: &str = "[Result \"1-0\"]\n[ECO \"C31\"]\n[ECOUrl \"https://www.chess.com/openings/Vienna-Game\"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0";

    fn draft_from(pgn: &str) -> GameDraft {
        let parsed = parse_single(pgn).unwrap();
        build_draft(
            &parsed,
            GameSource::Manual,
            "x".to_string(),
            "alice",
            None,
            None,
        )
    }

    #[test]
    fn position_win_ignores_a_conflicting_eco_header() {
        let draft = draft_from(VIENNA);
        assert_eq!(draft.eco.as_deref(), Some("C60"));
        assert!(draft.opening_name.unwrap().contains("Ruy Lopez"));
        assert_eq!(draft.opening_ply, Some(5));
        assert_eq!(draft.opening_source.as_deref(), Some("position"));
    }

    #[test]
    fn backfill_updates_store_and_is_idempotent() {
        let db = Db::in_memory().unwrap();
        let mut draft = draft_from(VIENNA);
        draft.external_id = "g1".to_string();
        draft.eco = Some("C31".to_string());
        draft.opening_name = Some("King's Gambit Declined".to_string());
        let id = db.with_tx(|tx| insert_game(tx, &draft)).unwrap().unwrap();
        let updated = backfill_openings(&db).unwrap();
        assert_eq!(updated, 1);
        let (eco, name): (String, String) = db
            .with(|conn| {
                Ok(conn.query_row(
                    "SELECT eco, opening_name FROM games WHERE id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(eco, "C60");
        assert!(name.contains("Ruy Lopez"));
        assert_eq!(backfill_openings(&db).unwrap(), 0);
    }

    #[test]
    fn header_fallback_is_used_only_without_positions() {
        let parsed =
            parse_single("[Result \"*\"]\n[ECO \"Z99\"]\n[Opening \"  Vienna   Game. \"]\n\n*")
                .unwrap();
        let opening = identify_opening(&parsed);
        assert_eq!(opening.eco.as_deref(), Some("Z99"));
        assert_eq!(opening.name.as_deref(), Some("Vienna Game"));
        assert_eq!(opening.source.as_deref(), Some("pgn_header"));
        assert_eq!(opening.ply, None);
    }

    #[test]
    fn user_color_matches_generated_draft() {
        let draft = draft_from(VIENNA);
        assert_eq!(draft.user_color, UserColor::None);
    }
}
