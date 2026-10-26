use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub phase: String,
    pub message: String,
    pub archives_done: u32,
    pub archives_total: u32,
    pub games_imported: u32,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub username: String,
    pub archives_processed: u32,
    pub games_imported: u32,
    pub games_skipped: u32,
}
