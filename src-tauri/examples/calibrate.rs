use std::path::Path;

use serde::Serialize;

use chenal_lib::domain::game::GameSource;
use chenal_lib::engine::EngineManager;
use chenal_lib::pgn::{build_draft, parse_single};
use chenal_lib::service::build::{build_analysis, BuildInput};
use chenal_lib::service::evaluation::evaluate_positions;

#[derive(Serialize)]
struct Row {
    id: i64,
    depth: u32,
    white: String,
    black: String,
    result: String,
    white_accuracy: Option<f64>,
    black_accuracy: Option<f64>,
    white_acpl: Option<f64>,
    black_acpl: Option<f64>,
    plies: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let db_path = args.next().unwrap_or_else(|| {
        std::env::var("HOME")
            .map(|home| format!("{home}/Library/Application Support/com.chenal.studio/chenal.db"))
            .unwrap_or_else(|_| "chenal.db".to_string())
    });
    let depth: u32 = args.next().and_then(|v| v.parse().ok()).unwrap_or(14);
    let ids: Vec<i64> = args.filter_map(|v| v.parse().ok()).collect();

    let con =
        rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let engine_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin");
    let engine = EngineManager::spawn(&engine_path).await?;

    for id in ids {
        let (pgn, white, black, result): (String, String, String, String) = con.query_row(
            "select pgn, white_username, black_username, result from games where id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let parsed = parse_single(&pgn)?;
        let first = parsed.moves.first().ok_or("no moves")?;
        let draft = build_draft(
            &parsed,
            GameSource::Chesscom,
            format!("calibrate-{id}"),
            &white,
            None,
            Some(&pgn),
        );
        let game = draft.clone().into_game(id);
        let mut fens = Vec::with_capacity(parsed.moves.len() + 1);
        fens.push(first.fen_before.clone());
        fens.extend(parsed.moves.iter().map(|mv| mv.fen_after.clone()));
        let evaluations = evaluate_positions(&engine, &fens, depth, 2, None, |_, _| {}).await?;
        let built = build_analysis(&BuildInput {
            game: &game,
            parsed: &parsed,
            evaluations: &evaluations,
            engine_name: engine.name(),
            engine_version: engine.version(),
            depth,
            multipv: 2,
        })?;
        let row = Row {
            id,
            depth,
            white,
            black,
            result,
            white_accuracy: built.analysis.white_accuracy,
            black_accuracy: built.analysis.black_accuracy,
            white_acpl: built.analysis.white_acpl,
            black_acpl: built.analysis.black_acpl,
            plies: parsed.moves.len(),
        };
        println!("{}", serde_json::to_string(&row)?);
    }
    Ok(())
}
