use std::path::Path;

use crate::db::connection::Db;
use crate::db::games_repo::{all_games, games_by_ids, insert_game};
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{GameSource, ImportSummary};
use crate::pgn::export::export_games;
use crate::pgn::{build_draft, parse_pgn};
use crate::service::hashing::sha256_hex;

pub fn import_pgn_text(db: &Db, text: &str, player_username: &str) -> AppResult<ImportSummary> {
    let report = parse_pgn(text);
    if report.games.is_empty() && !report.errors.is_empty() {
        return Err(AppError::Parse(
            report
                .errors
                .first()
                .cloned()
                .unwrap_or_else(|| "Ошибка PGN".to_string()),
        ));
    }
    let mut imported = 0u32;
    let mut skipped = 0u32;
    db.with_tx(|tx| {
        for parsed in &report.games {
            let pgn = parsed.to_pgn();
            let external_id = format!("manual-{}", sha256_hex(&pgn));
            let draft = build_draft(
                parsed,
                GameSource::Manual,
                external_id,
                player_username,
                None,
                None,
            );
            if insert_game(tx, &draft)?.is_some() {
                imported += 1;
            } else {
                skipped += 1;
            }
        }
        Ok(())
    })?;
    Ok(ImportSummary {
        imported,
        skipped,
        errors: report.errors,
    })
}

pub fn import_pgn_files(
    db: &Db,
    paths: &[String],
    player_username: &str,
) -> AppResult<ImportSummary> {
    let mut summary = ImportSummary {
        imported: 0,
        skipped: 0,
        errors: Vec::new(),
    };
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(text) => match import_pgn_text(db, &text, player_username) {
                Ok(result) => {
                    summary.imported += result.imported;
                    summary.skipped += result.skipped;
                    summary.errors.extend(result.errors);
                }
                Err(err) => summary.errors.push(format!("{path}: {err}")),
            },
            Err(err) => summary.errors.push(format!("{path}: {err}")),
        }
    }
    Ok(summary)
}

pub fn export_pgn(db: &Db, ids: &[i64]) -> AppResult<String> {
    let games = if ids.is_empty() {
        all_games(db)?
    } else {
        games_by_ids(db, ids)?
    };
    if games.is_empty() {
        return Err(AppError::NotFound("Нет партий для экспорта".to_string()));
    }
    Ok(export_games(&games))
}

pub fn export_pgn_to_path(db: &Db, ids: &[i64], path: &str) -> AppResult<()> {
    let pgn = export_pgn(db, ids)?;
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, pgn)?;
    Ok(())
}
