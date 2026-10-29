use serde::{Deserialize, Serialize};

use crate::domain::analysis::Phase;
use crate::domain::game::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorRecord {
    pub games: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubKpis {
    pub total_games: u32,
    pub analyzed_games: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub avg_accuracy: Option<f64>,
    pub avg_acpl: Option<f64>,
    pub avg_perf_elo: Option<f64>,
    pub white: ColorRecord,
    pub black: ColorRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningStat {
    pub eco: String,
    pub name: String,
    pub games: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub accuracy: Option<f64>,
    pub exit_eval: Option<f64>,
    pub exit_win_prob: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseStat {
    pub phase: Phase,
    pub accuracy: Option<f64>,
    pub moves: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccuracyPoint {
    pub game_id: i64,
    pub played_at: i64,
    pub accuracy: f64,
    pub opponent: String,
    pub color: Color,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeStats {
    pub time_trouble_moves: u32,
    pub impulsive_errors: u32,
    pub overthink_moves: u32,
    pub avg_time_spent: Option<f64>,
    pub pressure_accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubStats {
    pub kpis: HubKpis,
    pub openings: Vec<OpeningStat>,
    pub phases: Vec<PhaseStat>,
    pub trend: Vec<AccuracyPoint>,
    pub time: TimeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub available: bool,
    pub path: Option<String>,
    pub name: Option<String>,
    pub error: Option<String>,
}

pub const PIECE_SETS: [&str; 6] = [
    "cburnett",
    "merida",
    "chessnut",
    "rhosgfx",
    "celtic",
    "kiwen-suwi",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub player_username: String,
    pub analysis_depth: u32,
    pub multipv: u32,
    pub board_theme: String,
    pub piece_set: String,
    pub show_arrows: bool,
    pub auto_analyze: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            player_username: String::new(),
            analysis_depth: 14,
            multipv: 2,
            board_theme: "walnut".to_string(),
            piece_set: "cburnett".to_string(),
            show_arrows: true,
            auto_analyze: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningEntry {
    pub eco: String,
    pub name: String,
    pub pgn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub eco_openings: usize,
}
