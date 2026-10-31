use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{Mutex, Notify};

use crate::domain::error::AppResult;
use crate::engine::manager::EngineManager;

/// Stockfish process pool: one process per analysis job.
/// A lease (`PooledEngine`) returns the process to the pool when dropped.
pub struct EnginePool {
    managers: Vec<Arc<EngineManager>>,
    free: Mutex<Vec<usize>>,
    notify: Notify,
    path: PathBuf,
    size: usize,
    /// Global cancellation: set by `request_stop` and checked by jobs that have
    /// not started their block yet. Without it, blocks started after a stop
    /// would reset the engine flag and keep running.
    stop: AtomicBool,
}

pub struct PooledEngine {
    pool: Arc<EnginePool>,
    index: usize,
    manager: Arc<EngineManager>,
}

impl EnginePool {
    /// Each process: `Threads = 1`, `Hash = hash_mb`. Pool size is the number of
    /// concurrent games, not the number of threads inside the engine.
    pub async fn spawn(path: &Path, size: usize, hash_mb: u32) -> AppResult<Arc<Self>> {
        let size = size.clamp(1, 64);
        let mut managers = Vec::with_capacity(size);
        for _ in 0..size {
            match EngineManager::spawn_with(path, 1, hash_mb).await {
                Ok(manager) => managers.push(Arc::new(manager)),
                Err(err) => {
                    for manager in &managers {
                        manager.shutdown().await;
                    }
                    return Err(err);
                }
            }
        }
        let free = (0..size).collect();
        Ok(Arc::new(EnginePool {
            managers,
            free: Mutex::new(free),
            notify: Notify::new(),
            path: path.to_path_buf(),
            size,
            stop: AtomicBool::new(false),
        }))
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Waits for a free process. The queue is fair: a released process goes to
    /// the first waiter.
    pub async fn acquire(self: &Arc<Self>) -> PooledEngine {
        loop {
            let index = self.free.lock().await.pop();
            if let Some(index) = index {
                return PooledEngine {
                    pool: Arc::clone(self),
                    index,
                    manager: Arc::clone(&self.managers[index]),
                };
            }
            self.notify.notified().await;
        }
    }

    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        for manager in &self.managers {
            manager.request_stop();
        }
    }

    /// New analysis job: the previous cancellation no longer applies.
    pub fn clear_stop(&self) {
        self.stop.store(false, Ordering::SeqCst);
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub async fn shutdown(&self) {
        for manager in &self.managers {
            manager.shutdown().await;
        }
    }

    pub fn name(&self) -> &str {
        self.managers
            .first()
            .map_or("Stockfish", |manager| manager.name())
    }

    pub fn version(&self) -> &str {
        self.managers
            .first()
            .map_or("unknown", |manager| manager.version())
    }
}

impl PooledEngine {
    pub fn manager(&self) -> &EngineManager {
        &self.manager
    }
}

impl Drop for PooledEngine {
    fn drop(&mut self) {
        let pool = Arc::clone(&self.pool);
        let index = self.index;
        tokio::spawn(async move {
            pool.free.lock().await.push(index);
            pool.notify.notify_one();
        });
    }
}

/// Number of analysis processes. One Stockfish thread per process is better than
/// several threads inside one process: at depth 14 a second thread yields twice
/// as many nodes, but total time grows, so parallelism comes from the number of
/// processes. Measured throughput on a 4P+6E (M5): 3 processes — 1.8x, 6 — 2.7x,
/// 8 — 2.8x, 10 — 2.5x; hence "all performance cores plus two thirds of the
/// efficiency cores".
pub fn default_workers() -> usize {
    let performance = sysctl_usize("hw.perflevel0.logicalcpu");
    let efficiency = sysctl_usize("hw.perflevel1.logicalcpu").unwrap_or(0);
    let workers = match performance {
        Some(performance) => performance + (efficiency * 2) / 3,
        None => std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1),
    };
    workers.clamp(1, 64)
}

fn sysctl_usize(key: &str) -> Option<usize> {
    std::process::Command::new("sysctl")
        .args(["-n", key])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

/// Hash size per process: at most 128 MB and at least 16 MB.
pub fn hash_mb_for(workers: usize) -> u32 {
    let per_worker = 1024 / u32::try_from(workers.max(1)).unwrap_or(8);
    per_worker.clamp(16, 128)
}

pub fn workers_from_setting(setting: u32) -> usize {
    let maximum = default_workers();
    if setting == 0 {
        maximum
    } else {
        usize::try_from(setting)
            .unwrap_or(maximum)
            .clamp(1, maximum)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn auto_workers_is_at_least_one() {
        assert!(default_workers() >= 1);
    }

    #[test]
    fn setting_zero_means_auto() {
        assert_eq!(workers_from_setting(0), default_workers());
    }

    #[test]
    fn explicit_setting_is_clamped() {
        assert_eq!(workers_from_setting(1), 1);
        assert!(workers_from_setting(10_000) <= default_workers());
    }

    #[test]
    fn hash_stays_within_bounds() {
        for workers in 1..=16 {
            let hash = hash_mb_for(workers);
            assert!((16..=128).contains(&hash), "{workers}: {hash}");
        }
    }

    #[tokio::test]
    async fn pool_returns_engine_back_on_drop() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries/stockfish-aarch64-apple-darwin");
        if !path.is_file() {
            return;
        }
        let pool = EnginePool::spawn(&path, 1, 16).await.unwrap();
        let first = pool.acquire().await;
        assert_eq!(first.pool.size(), 1);
        drop(first);
        let second = tokio::time::timeout(std::time::Duration::from_secs(2), pool.acquire())
            .await
            .expect("engine returned to pool");
        drop(second);
        pool.shutdown().await;
    }

    #[tokio::test]
    async fn second_acquire_waits_for_the_first() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries/stockfish-aarch64-apple-darwin");
        if !path.is_file() {
            return;
        }
        let pool = EnginePool::spawn(&path, 1, 16).await.unwrap();
        let first = pool.acquire().await;
        let pending =
            tokio::time::timeout(std::time::Duration::from_millis(200), pool.acquire()).await;
        assert!(pending.is_err(), "second lease must wait");
        drop(first);
        let second = tokio::time::timeout(std::time::Duration::from_secs(2), pool.acquire())
            .await
            .expect("acquired after release");
        drop(second);
        pool.shutdown().await;
    }
}
