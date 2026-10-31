use serde::{Deserialize, Serialize};

use crate::domain::analysis::Phase;
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{Color, GameSource};

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
    /// Spread (standard deviation) of the Performance Elo estimate, if shown.
    pub avg_perf_elo_spread: Option<f64>,
    pub white: ColorRecord,
    pub black: ColorRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformRating {
    /// Account the games belong to; NULL — manual PGNs without an account.
    pub account_id: Option<i64>,
    pub account_username: Option<String>,
    /// Account platform. For games without an account — the PGN source.
    pub platform: GameSource,
    pub games: u32,
    pub analyzed: u32,
    pub avg_user_elo: Option<f64>,
    pub avg_opponent_elo: Option<f64>,
    /// Average Performance Elo only within this platform/account.
    pub perf_elo: Option<f64>,
    pub perf_elo_spread: Option<f64>,
    /// How many games produced a Performance Elo value.
    pub perf_elo_games: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningStat {
    pub eco: String,
    pub name: String,
    pub games: u32,
    /// How many games of this opening have already been analyzed.
    pub analyzed: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub accuracy: Option<f64>,
    pub exit_eval: Option<f64>,
    pub exit_win_prob: Option<f64>,
    /// Opening price: games × average Win% loss on the player's moves.
    pub price: f64,
    /// Accuracy can be shown only when n ≥ 5.
    pub reliable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseStat {
    pub phase: Phase,
    pub accuracy: Option<f64>,
    pub moves: u32,
    /// Number of analyzed games that have moves in this phase.
    pub games: u32,
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

/// Configuration the hub aggregates were built from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisConfigView {
    pub engine_name: String,
    pub engine_version: String,
    pub depth: i32,
    pub multipv: i32,
    pub classifier_version: String,
    pub accuracy_method_version: String,
    /// How many analyses of this configuration went into the aggregates.
    pub included: u32,
    /// How many analyses exist in the database in total.
    pub total: u32,
    /// Games analyzed with a different configuration — they can be recalculated.
    pub mismatched_game_ids: Vec<i64>,
    /// Subset of `mismatched_game_ids` that is shallower than the current settings.
    /// Only these games are worth recalculating.
    pub upgrade_game_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubStats {
    pub kpis: HubKpis,
    pub ratings: Vec<PlatformRating>,
    pub openings: Vec<OpeningStat>,
    pub phases: Vec<PhaseStat>,
    pub trend: Vec<AccuracyPoint>,
    pub time: TimeStats,
    pub config: Option<AnalysisConfigView>,
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

pub const BOARD_THEMES: [&str; 3] = ["midnight", "walnut", "slate"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Legacy profile for manual PGNs without a selected account.
    pub player_username: String,
    /// NULL — "all accounts"; otherwise the selected account.
    pub active_account_id: Option<i64>,
    pub analysis_depth: u32,
    pub multipv: u32,
    pub board_theme: String,
    pub piece_set: String,
    pub show_arrows: bool,
    pub auto_analyze: bool,
    /// 0 — auto (performance cores plus some efficiency cores), otherwise 1…N processes.
    pub analysis_threads: u32,
    /// Resume an unfinished queue when the app starts.
    pub auto_resume_analysis: bool,
    pub language: String,
}

impl AppSettings {
    /// Validates all settings at once. `set_many` must not bypass the checks
    /// that `set_setting` applies to individual keys.
    pub fn validate(&self) -> AppResult<()> {
        if !(6..=24).contains(&self.analysis_depth) {
            return Err(AppError::Validation(
                "Analysis depth must be between 6 and 24".to_string(),
            ));
        }
        if !(1..=3).contains(&self.multipv) {
            return Err(AppError::Validation(
                "MultiPV must be between 1 and 3".to_string(),
            ));
        }
        if self.analysis_threads > 64 {
            return Err(AppError::Validation(
                "Analysis workers must be 0 (auto) or between 1 and 64".to_string(),
            ));
        }
        if !PIECE_SETS.contains(&self.piece_set.as_str()) {
            return Err(AppError::Validation(format!(
                "Unknown piece set «{}»",
                self.piece_set
            )));
        }
        if !BOARD_THEMES.contains(&self.board_theme.as_str()) {
            return Err(AppError::Validation(format!(
                "Unknown board theme «{}»",
                self.board_theme
            )));
        }
        if !(2..=5).contains(&self.language.len())
            || !self.language.chars().all(|c| c.is_ascii_lowercase())
        {
            return Err(AppError::Validation(format!(
                "Invalid language code «{}»",
                self.language
            )));
        }
        Ok(())
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            player_username: String::new(),
            active_account_id: None,
            analysis_depth: 14,
            multipv: 2,
            board_theme: "walnut".to_string(),
            piece_set: "cburnett".to_string(),
            show_arrows: true,
            auto_analyze: true,
            analysis_threads: 0,
            auto_resume_analysis: false,
            language: "en".to_string(),
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
    pub max_analysis_workers: u32,
    pub db_path: String,
    pub log_path: String,
}
