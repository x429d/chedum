//! Служебная консоль для операций над локальной базой Chenal.
//! Не входит в бандл приложения; используется для аудита и пакетного разбора.

use std::path::PathBuf;
use std::process::ExitCode;

use chenal_lib::db::connection::Db;
use chenal_lib::domain::error::AppResult;
use chenal_lib::service::openings;

fn default_db() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library/Application Support/com.chenal.studio/chenal.db")
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chenal-cli: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().cloned().unwrap_or_else(|| "audit".to_string());
    let db_path = args
        .iter()
        .position(|value| value == "--db")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or_else(default_db);
    let db = Db::open(&db_path)?;
    match command.as_str() {
        "audit" => audit(&db),
        "openings" => {
            let updated = openings::backfill_openings(&db)?;
            println!("openings: обновлено партий — {updated}");
            audit(&db)
        }
        "backfill" => {
            let summaries = chenal_lib::db::analyses_repo::backfill_summaries(&db)?;
            let moments = chenal_lib::db::analyses_repo::backfill_key_moments(&db)?;
            let openings = openings::backfill_openings(&db)?;
            println!("summaries: {summaries}, key moments: {moments}, openings: {openings}");
            audit(&db)
        }
        "hub" => hub(&db),
        other => {
            eprintln!("Неизвестная команда «{other}»: audit | openings | backfill | hub");
            Ok(())
        }
    }
}

fn audit(db: &Db) -> AppResult<()> {
    db.with(|conn| {
        let games: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))?;
        let analyses: i64 =
            conn.query_row("SELECT COUNT(*) FROM game_analyses", [], |row| row.get(0))?;
        println!("games: {games}");
        println!("analyses: {analyses}");
        let mut stmt = conn.prepare(
            "SELECT engine_name, depth, multipv, COUNT(*) FROM game_analyses
             GROUP BY engine_name, depth, multipv ORDER BY COUNT(*) DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (engine, depth, multipv, count) = row?;
            println!("config: {engine} · depth {depth} · MultiPV {multipv} — {count}");
        }
        Ok(())
    })
}

fn hub(db: &Db) -> AppResult<()> {
    let stats = chenal_lib::db::stats_repo::hub_stats(db)?;
    if let Some(config) = &stats.config {
        println!(
            "config: {} {} · depth {} · MultiPV {} · classifier {} · accuracy {} · included {} of {}",
            config.engine_name,
            config.engine_version,
            config.depth,
            config.multipv,
            config.classifier_version,
            config.accuracy_method_version,
            config.included,
            config.total
        );
        println!("mismatched: {:?}", config.mismatched_game_ids);
    }
    println!(
        "kpis: total={} analyzed={} accuracy={:?} acpl={:?} perf={:?}",
        stats.kpis.total_games,
        stats.kpis.analyzed_games,
        stats.kpis.avg_accuracy,
        stats.kpis.avg_acpl,
        stats.kpis.avg_perf_elo
    );
    println!("trend points: {}", stats.trend.len());
    if let (Some(first), Some(last)) = (stats.trend.first(), stats.trend.last()) {
        println!(
            "trend: first {} at {}, last {} at {}",
            first.accuracy, first.played_at, last.accuracy, last.played_at
        );
    }
    for opening in stats.openings.iter().take(5) {
        println!(
            "opening: {} {} games={} analyzed={} acc={:?} price={:.1} reliable={}",
            opening.eco,
            opening.name,
            opening.games,
            opening.analyzed,
            opening.accuracy,
            opening.price,
            opening.reliable
        );
    }
    Ok(())
}
