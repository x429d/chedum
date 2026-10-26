use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineLine {
    pub rank: u32,
    pub cp: Option<i32>,
    pub mate: Option<i32>,
    pub win_prob: f64,
    pub uci_moves: Vec<String>,
    pub san_moves: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineUpdate {
    pub fen: String,
    pub depth: u32,
    pub seldepth: Option<u32>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub lines: Vec<EngineLine>,
    pub best_move: Option<String>,
    pub best_san: Option<String>,
    pub finished: bool,
}
