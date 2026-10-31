use super::*;

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn bundled_engine() -> Option<PathBuf> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/stockfish-aarch64-apple-darwin");
    path.is_file().then_some(path)
}

#[tokio::test]
async fn analyzes_start_position_with_bundled_stockfish() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = EngineManager::spawn(&path).await.unwrap();
    let _job = manager.begin_job().await.unwrap();
    let mut updates = 0u32;
    let update = manager
        .analyze(START, 8, 2, |_| updates += 1)
        .await
        .unwrap();
    assert!(update.finished);
    assert!(!update.lines.is_empty());
    assert!(update.best_move.is_some());
    assert!(update.best_san.is_some());
    // The final update must contain SAN for all lines: analysis stores
    // best_san and the PV from it, not from intermediate updates.
    assert!(update
        .lines
        .iter()
        .all(|line| !line.san_moves.is_empty() && line.san_moves.len() == line.uci_moves.len()));
    assert!(updates > 0);
    assert!(update.depth >= 8);
    manager.shutdown().await;
}

#[tokio::test]
async fn prepare_game_clears_state_without_desyncing_uci() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = EngineManager::spawn(&path).await.unwrap();
    let _job = manager.begin_job().await.unwrap();
    manager.analyze(START, 6, 1, |_| {}).await.unwrap();
    manager.prepare_game().await.unwrap();
    let update = manager.analyze(START, 6, 1, |_| {}).await.unwrap();
    assert!(update.finished);
    assert!(!update.lines.is_empty());
    manager.shutdown().await;
}

#[tokio::test]
async fn rejects_second_concurrent_job() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = EngineManager::spawn(&path).await.unwrap();
    let job = manager.begin_job().await.unwrap();
    let second = manager.begin_job().await;
    assert!(matches!(second, Err(AppError::Busy(_))));
    drop(job);
    assert!(manager.begin_job().await.is_ok());
    manager.shutdown().await;
}

#[tokio::test]
async fn stop_cancels_a_running_search() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = std::sync::Arc::new(EngineManager::spawn(&path).await.unwrap());
    let _job = manager.begin_job().await.unwrap();
    let stopper = tokio::spawn({
        let manager = std::sync::Arc::clone(&manager);
        async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            manager.request_stop();
        }
    });
    let result = timeout(
        Duration::from_secs(8),
        manager.analyze(START, 30, 1, |_| {}),
    )
    .await
    .expect("search stopped");
    assert!(matches!(result, Err(AppError::Engine(_))));
    stopper.await.unwrap();
    manager.shutdown().await;
}

#[tokio::test]
async fn recovers_after_engine_process_is_killed() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = EngineManager::spawn(&path).await.unwrap();
    let job = manager.begin_job().await.unwrap();
    // Simulates a worker crash: the process is gone, the job is still alive.
    manager.shutdown().await;
    drop(job);
    let _job = manager.begin_job().await.unwrap();
    let update = timeout(
        Duration::from_secs(20),
        manager.analyze(START, 8, 1, |_| {}),
    )
    .await
    .expect("engine restarted")
    .unwrap();
    assert!(update.finished);
    assert!(!update.lines.is_empty());
    manager.shutdown().await;
}

#[tokio::test]
async fn new_job_clears_previous_cancellation() {
    let Some(path) = bundled_engine() else {
        return;
    };
    let manager = EngineManager::spawn(&path).await.unwrap();
    manager.request_stop();
    assert!(manager.is_cancelled());
    let _job = manager.begin_job().await.unwrap();
    assert!(!manager.is_cancelled());
    manager.shutdown().await;
}
