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
        other => {
            eprintln!("Неизвестная команда «{other}»: audit | openings");
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
