use std::path::Path;

use serde::Serialize;

use chenal_lib::domain::analysis::{GameAnalysis, MoveEvaluation};
use chenal_lib::domain::game::{GameDraft, GameSource};
use chenal_lib::engine::EngineManager;
use chenal_lib::pgn::{build_draft, parse_single};
use chenal_lib::service::build::{build_analysis, BuildInput};
use chenal_lib::service::evaluation::evaluate_positions;

#[derive(Serialize)]
struct SeedEntry {
    game: GameDraft,
    analysis: GameAnalysis,
    moves: Vec<MoveEvaluation>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let depth = 14;
    let engine_path = Path::new("binaries/stockfish-aarch64-apple-darwin");
    let engine = EngineManager::spawn(engine_path).await?;
    let games: [(&str, &str); 4] = [
        ("demo-opera", include_str!("../resources/demo_opera.pgn")),
        (
            "demo-immortal",
            include_str!("../resources/demo_immortal.pgn"),
        ),
        (
            "demo-evergreen",
            include_str!("../resources/demo_evergreen.pgn"),
        ),
        ("demo-legal", include_str!("../resources/demo_legal.pgn")),
    ];
    let mut entries = Vec::new();
    for (external_id, pgn) in games {
        let parsed = parse_single(pgn)?;
        let first = parsed.moves.first().ok_or("demo game has no moves")?;
        let draft = build_draft(
            &parsed,
            GameSource::Chesscom,
            external_id.to_string(),
            "chenal_player",
            None,
            Some(pgn),
        );
        let game = draft.clone().into_game(0);
        let mut fens = Vec::with_capacity(parsed.moves.len() + 1);
        fens.push(first.fen_before.clone());
        fens.extend(parsed.moves.iter().map(|mv| mv.fen_after.clone()));
        eprintln!(
            "{external_id}: {} позиций, анализ depth={depth}",
            fens.len()
        );
        let evaluations = evaluate_positions(&engine, &fens, depth, 2, |done, total| {
            if done == total || done % 10 == 0 {
                eprintln!("  {done}/{total}");
            }
        })
        .await?;
        let mut built = build_analysis(&BuildInput {
            game: &game,
            parsed: &parsed,
            evaluations: &evaluations,
            engine_name: engine.name(),
            engine_version: engine.version(),
            depth,
            multipv: 2,
        })?;
        built.analysis.game_id = 0;
        for mv in &mut built.moves {
            mv.game_id = 0;
        }
        entries.push(SeedEntry {
            game: draft,
            analysis: built.analysis,
            moves: built.moves,
        });
    }
    let json = serde_json::to_string_pretty(&entries)?;
    std::fs::write("resources/demo_seed.json", json)?;
    engine.shutdown().await;
    eprintln!("demo_seed.json: {} партий", entries.len());
    Ok(())
}
