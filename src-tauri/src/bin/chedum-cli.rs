//! Internal console for operations on the local Chedum database.
//! Not shipped in the app bundle; used for audits and batch analysis.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use chedum_lib::db::connection::Db;
use chedum_lib::domain::error::AppResult;
use chedum_lib::engine::pool::{hash_mb_for, workers_from_setting, EnginePool};
use chedum_lib::service::openings;

fn default_db() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join("Library/Application Support/com.chedum.studio/chedum.db")
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chedum-cli: {err}");
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
            println!("openings: updated games — {updated}");
            audit(&db)
        }
        "backfill" => {
            let fen_accuracy = chedum_lib::db::analyses_repo::backfill_fen_accuracy(&db)?;
            let summaries = chedum_lib::db::analyses_repo::backfill_summaries(&db)?;
            let moments = chedum_lib::db::analyses_repo::backfill_key_moments(&db)?;
            let openings = openings::backfill_openings(&db)?;
            println!(
                "fen accuracy: {fen_accuracy}, summaries: {summaries}, key moments: {moments}, openings: {openings}"
            );
            audit(&db)
        }
        "hub" => hub(&db),
        "analyze" => block_on(analyze(&db, &args)),
        other => {
            eprintln!("Unknown command «{other}»: audit | openings | backfill | hub | analyze");
            Ok(())
        }
    }
}

fn block_on<F>(future: F) -> AppResult<()>
where
    F: std::future::Future<Output = AppResult<()>>,
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| chedum_lib::domain::error::AppError::Internal(err.to_string()))?;
    runtime.block_on(future)
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|value| value == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn engine_path() -> PathBuf {
    if let Ok(path) = std::env::var("CHEDUM_STOCKFISH") {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin")
}

async fn analyze(db: &Db, args: &[String]) -> AppResult<()> {
    let settings = chedum_lib::db::settings_repo::get_settings(db)?;
    let depth = arg_value(args, "--depth")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(settings.analysis_depth);
    let multipv = arg_value(args, "--multipv")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(settings.multipv);
    let limit = arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20);
    let setting = arg_value(args, "--workers")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let workers = workers_from_setting(setting);
    let explicit_ids: Option<Vec<i64>> = arg_value(args, "--ids").map(|value| {
        value
            .split(',')
            .filter_map(|item| item.trim().parse::<i64>().ok())
            .collect()
    });
    let fresh = args.iter().any(|value| value == "--fresh");
    let parallel = args.iter().any(|value| value == "--parallel");
    if fresh {
        if let Some(ids) = &explicit_ids {
            for id in ids {
                db.with(|conn| {
                    conn.execute("DELETE FROM game_analyses WHERE game_id = ?1", [id])?;
                    Ok(())
                })?;
            }
        }
    }
    let ids: Vec<i64> = match explicit_ids {
        Some(ids) => ids,
        None => {
            let key = chedum_lib::service::queue::target_config_key(db, &settings)?;
            let mut plan = chedum_lib::db::games_repo::pending_analysis(
                db,
                &chedum_lib::domain::game::GameFilter::default(),
                &key,
                settings.analysis_depth,
                settings.multipv,
            )?;
            plan.ids.truncate(limit);
            if plan.excluded > 0 {
                println!("excluded after 3 errors: {}", plan.excluded);
            }
            plan.ids
        }
    };
    if ids.is_empty() {
        println!("No games to analyze");
        return Ok(());
    }
    let path = engine_path();
    if !path.is_file() {
        eprintln!("Stockfish not found: {}", path.display());
        return Ok(());
    }
    println!(
        "analyze: {} games, depth {depth}, MultiPV {multipv}, workers {workers}, hash {} MB",
        ids.len(),
        hash_mb_for(workers)
    );
    let pool = EnginePool::spawn(&path, workers, hash_mb_for(workers)).await?;
    let started = Instant::now();
    let next = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut tasks = Vec::new();
    for _ in 0..workers {
        let pool = Arc::clone(&pool);
        let next = Arc::clone(&next);
        let ids = ids.clone();
        let db = db.clone();
        tasks.push(tokio::spawn(async move {
            let mut results = Vec::new();
            loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if index >= ids.len() {
                    break;
                }
                let id = ids[index];
                let lease = pool.acquire().await;
                let game_started = Instant::now();
                let outcome = if parallel {
                    // One game's positions are spread across the whole pool:
                    // only worthwhile with a small number of games.
                    chedum_lib::service::analysis::analyze_game_parallel(
                        &db, &pool, id, depth, multipv, None,
                    )
                    .await
                } else {
                    chedum_lib::service::analysis::analyze_game(
                        &db,
                        lease.manager(),
                        id,
                        depth,
                        multipv,
                        None,
                    )
                    .await
                };
                let elapsed = game_started.elapsed().as_secs_f64();
                results.push((id, elapsed, outcome.map(|detail| detail.analysis)));
            }
            results
        }));
    }
    let mut games = 0usize;
    let mut failures = 0usize;
    for task in tasks {
        match task.await {
            Ok(results) => {
                for (id, elapsed, outcome) in results {
                    games += 1;
                    match outcome {
                        Ok(Some(analysis)) => println!(
                            "  #{id}: {:.1}s white_acc={:?} black_acc={:?} white_acpl={:?} black_acpl={:?}",
                            elapsed,
                            analysis.white_accuracy,
                            analysis.black_accuracy,
                            analysis.white_acpl,
                            analysis.black_acpl
                        ),
                        Ok(None) => println!("  #{id}: {:.1}s (no analysis)", elapsed),
                        Err(err) => {
                            failures += 1;
                            println!("  #{id}: {:.1}s error: {err}", elapsed);
                        }
                    }
                }
            }
            Err(err) => {
                failures += 1;
                eprintln!("worker join error: {err}");
            }
        }
    }
    pool.shutdown().await;
    println!(
        "done: {games} processed, {failures} errors, {:.1}s total",
        started.elapsed().as_secs_f64()
    );
    Ok(())
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
    let stats = chedum_lib::db::stats_repo::hub_stats(db, None)?;
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
        println!("upgrade: {:?}", config.upgrade_game_ids);
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
    for rating in &stats.ratings {
        println!(
            "rating: {} {} games={} analyzed={} your={:?} opponent={:?} perf={:?}±{:?}",
            rating.platform.as_str(),
            rating.account_username.as_deref().unwrap_or("(no account)"),
            rating.games,
            rating.analyzed,
            rating.avg_user_elo,
            rating.avg_opponent_elo,
            rating.perf_elo,
            rating.perf_elo_spread
        );
    }
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
