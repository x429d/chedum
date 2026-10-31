use std::sync::Arc;

use crate::db::analyses_repo::checkpoints::{self, CheckpointKey};
use crate::db::connection::Db;
use crate::domain::error::AppResult;
use crate::service::evaluation::PositionEval;

/// Durable per-block evaluation cache. Blocks are the unit of parallel work,
/// so resuming a cancelled game never mixes warm and cold search results.
#[derive(Clone)]
pub struct Checkpoint {
    db: Db,
    game_id: i64,
    depth: u32,
    multipv: u32,
    engine_name: Arc<str>,
    engine_version: Arc<str>,
}

impl Checkpoint {
    pub fn new(
        db: &Db,
        game_id: i64,
        depth: u32,
        multipv: u32,
        engine_name: &str,
        engine_version: &str,
    ) -> Self {
        Checkpoint {
            db: db.clone(),
            game_id,
            depth,
            multipv,
            engine_name: Arc::from(engine_name),
            engine_version: Arc::from(engine_version),
        }
    }

    fn key(&self) -> CheckpointKey<'_> {
        CheckpointKey {
            game_id: self.game_id,
            depth: self.depth,
            multipv: self.multipv,
            engine_name: &self.engine_name,
            engine_version: &self.engine_version,
        }
    }

    /// Drops checkpoints left by another engine or search configuration.
    pub fn prepare(&self) -> AppResult<()> {
        checkpoints::clear_stale(&self.db, &self.key())
    }

    pub fn load_block(&self, block_index: usize) -> AppResult<Option<Vec<PositionEval>>> {
        let Some(json) = checkpoints::load_block(&self.db, &self.key(), block_index)? else {
            return Ok(None);
        };
        Ok(serde_json::from_str(&json).ok())
    }

    pub fn store_block(&self, block_index: usize, evaluations: &[PositionEval]) -> AppResult<()> {
        let json = serde_json::to_string(evaluations)?;
        checkpoints::store_block(&self.db, &self.key(), block_index, &json)
    }

    pub fn clear(&self) -> AppResult<()> {
        checkpoints::clear_game(&self.db, self.game_id)
    }
}
