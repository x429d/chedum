use serde::{Deserialize, Serialize};

use crate::domain::account::AccountPlatform;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    /// Stable phase key: `archives`, `archive`, `download`, `parse`, `import`,
    /// `done`. The frontend localizes it and ignores `message`.
    pub phase: String,
    /// Technical detail retained for logs and older clients.
    pub message: String,
    pub archives_done: u32,
    pub archives_total: u32,
    pub games_imported: u32,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub account_id: i64,
    pub platform: AccountPlatform,
    pub username: String,
    pub archives_processed: u32,
    pub games_imported: u32,
    /// Games that were already present in the local library.
    pub duplicates: u32,
    /// Games intentionally excluded: bot games, unsupported variants, empty PGN.
    pub filtered: u32,
    /// Human-readable reasons for games that could not be parsed.
    pub errors: Vec<String>,
}
