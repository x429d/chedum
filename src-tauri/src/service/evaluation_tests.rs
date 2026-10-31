use std::path::Path;

use super::*;

fn bundled_engine() -> Option<std::path::PathBuf> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin");
    path.is_file().then_some(path)
}

/// Same positions on pools of different sizes: blocks start with a clean
/// table, so the number of processes must change neither order nor evaluations.
#[tokio::test]
async fn pooled_evaluation_is_order_and_pool_size_independent() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        "r1bqkbnr/pp1ppppp/2n5/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        "r1bqkbnr/pp1ppppp/2n5/2p5/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        "r1bqkbnr/pp2pppp/2n5/2pp4/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 4",
        "r1bqkbnr/pp2pppp/2n5/3p4/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4",
        "r1bqkbnr/pp2pppp/2n5/3P4/2B5/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 4",
        "r1bqkbnr/pp3ppp/2n5/3p4/2B5/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 5",
    ]
    .map(String::from);
    let small = EnginePool::spawn(&path, 2, 16).await.unwrap();
    let large = EnginePool::spawn(&path, 3, 16).await.unwrap();
    let first = evaluate_positions_pooled(&small, &fens, 6, 1, None, Arc::new(|_| {}))
        .await
        .unwrap();
    let second = evaluate_positions_pooled(&large, &fens, 6, 1, None, Arc::new(|_| {}))
        .await
        .unwrap();
    small.shutdown().await;
    large.shutdown().await;

    assert_eq!(first.len(), fens.len());
    for (index, evaluation) in first.iter().enumerate() {
        assert_eq!(evaluation.fen, fens[index], "position order");
        let other = &second[index];
        assert_eq!(evaluation.best_move, other.best_move, "position {index}");
        assert_eq!(
            evaluation.lines.first().map(|line| line.score_cp),
            other.lines.first().map(|line| line.score_cp),
            "позиция {index}"
        );
    }
}

#[tokio::test]
async fn stop_cancels_parallel_blocks() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let fens: Vec<String> = (0..16)
        .map(|_| "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
        .collect();
    let pool = EnginePool::spawn(&path, 2, 16).await.unwrap();
    let stopper = {
        let pool = Arc::clone(&pool);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(60)).await;
            pool.request_stop();
        })
    };
    let result = evaluate_positions_pooled(&pool, &fens, 18, 1, None, Arc::new(|_| {})).await;
    stopper.await.unwrap();
    pool.shutdown().await;
    assert!(result.is_err(), "остановка должна прерывать блоки");
}
