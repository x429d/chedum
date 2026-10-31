use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::game::Color;

pub use crate::domain::move_category::MoveCategory;
pub use crate::domain::phase::{Phase, TimeFlag};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PvLine {
    pub rank: u32,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub win_prob: f64,
    pub uci_moves: Vec<String>,
    pub san_moves: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveEvaluation {
    pub game_id: i64,
    pub ply: i32,
    /// Side that made the move. Stored explicitly because `ply % 2` is wrong
    /// for games started from a FEN position.
    pub player_color: Color,
    pub phase: Phase,
    pub fen_before: String,
    pub fen_after: String,
    pub played_uci: String,
    pub played_san: String,
    pub best_uci: Option<String>,
    pub best_san: Option<String>,
    pub eval_cp: Option<i32>,
    pub eval_mate: Option<i32>,
    pub best_eval_cp: Option<i32>,
    pub best_eval_mate: Option<i32>,
    pub win_prob_white: f64,
    pub win_prob_loss: f64,
    pub move_accuracy: f64,
    pub classification: MoveCategory,
    pub is_critical: bool,
    #[serde(default)]
    pub is_key_moment: bool,
    pub material_diff_white: i32,
    pub clock_remaining_sec: Option<f64>,
    pub time_spent_sec: Option<f64>,
    pub time_flag: TimeFlag,
    pub explanation_tag: Option<String>,
    /// Localization parameters for `explanation_tag`. New analyses render the
    /// explanation in the UI language; `explanation_text` is a legacy fallback.
    #[serde(default)]
    pub explanation_params: Option<BTreeMap<String, String>>,
    pub explanation_text: Option<String>,
    pub pv_lines: Vec<PvLine>,
    pub user_comment: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCounts {
    pub white: BTreeMap<MoveCategory, u32>,
    pub black: BTreeMap<MoveCategory, u32>,
}

impl CategoryCounts {
    pub fn new() -> Self {
        let mut counts = CategoryCounts::default();
        for category in MoveCategory::ALL {
            counts.white.insert(category, 0);
            counts.black.insert(category, 0);
        }
        counts
    }

    pub fn increment(&mut self, color: Color, category: MoveCategory) {
        let map = match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
        *map.entry(category).or_insert(0) += 1;
    }

    pub fn get(&self, color: Color, category: MoveCategory) -> u32 {
        let map = match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        };
        map.get(&category).copied().unwrap_or(0)
    }
}

/// Methodology versions attached to each analysis.
pub const CLASSIFIER_VERSION: &str = "classifier-v1";
pub const ACCURACY_METHOD_VERSION: &str = "lichess-v1";
pub const DEFAULT_ENGINE_VERSION: &str = "19";

fn default_engine_version() -> String {
    DEFAULT_ENGINE_VERSION.to_string()
}

fn default_classifier_version() -> String {
    CLASSIFIER_VERSION.to_string()
}

fn default_accuracy_method() -> String {
    ACCURACY_METHOD_VERSION.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameAnalysis {
    pub game_id: i64,
    pub engine_name: String,
    #[serde(default = "default_engine_version")]
    pub engine_version: String,
    #[serde(default = "default_classifier_version")]
    pub classifier_version: String,
    #[serde(default = "default_accuracy_method")]
    pub accuracy_method_version: String,
    pub depth: i32,
    pub multipv: i32,
    pub analyzed_at: String,
    pub white_accuracy: Option<f64>,
    pub black_accuracy: Option<f64>,
    pub white_acpl: Option<f64>,
    pub black_acpl: Option<f64>,
    pub white_perf_elo: Option<i32>,
    pub black_perf_elo: Option<i32>,
    pub white_opening_acc: Option<f64>,
    pub white_middlegame_acc: Option<f64>,
    pub white_endgame_acc: Option<f64>,
    pub black_opening_acc: Option<f64>,
    pub black_middlegame_acc: Option<f64>,
    pub black_endgame_acc: Option<f64>,
    pub counts: CategoryCounts,
    pub summary_text: String,
    pub key_moments_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisDraft {
    pub analysis: GameAnalysis,
    pub moves: Vec<MoveEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisProgress {
    pub stage: String,
    pub ply: u32,
    pub total: u32,
    pub percent: f64,
    pub message: String,
}
