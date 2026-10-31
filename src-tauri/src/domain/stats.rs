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
    /// Разброс (стандартное отклонение) оценки Performance Elo, если она показана.
    pub avg_perf_elo_spread: Option<f64>,
    pub white: ColorRecord,
    pub black: ColorRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningStat {
    pub eco: String,
    pub name: String,
    pub games: u32,
    /// Сколько партий этого дебюта уже разобрано.
    pub analyzed: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub accuracy: Option<f64>,
    pub exit_eval: Option<f64>,
    pub exit_win_prob: Option<f64>,
    /// Цена дебюта: партии × средняя потеря Win% на ходах игрока.
    pub price: f64,
    /// Точность можно показывать только при n ≥ 5.
    pub reliable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseStat {
    pub phase: Phase,
    pub accuracy: Option<f64>,
    pub moves: u32,
    /// Число разобранных партий, у которых есть ходы в этой фазе.
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

/// Конфигурация, по которой собраны агрегаты хаба.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisConfigView {
    pub engine_name: String,
    pub engine_version: String,
    pub depth: i32,
    pub multipv: i32,
    pub classifier_version: String,
    pub accuracy_method_version: String,
    /// Сколько анализов этой конфигурации вошло в агрегаты.
    pub included: u32,
    /// Сколько анализов всего в базе.
    pub total: u32,
    /// Партии с анализом другой конфигурации — их можно пересчитать.
    pub mismatched_game_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HubStats {
    pub kpis: HubKpis,
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
    /// 0 — авто (производительные ядра плюс часть энергоэффективных), иначе 1…N процессов.
    pub analysis_threads: u32,
    /// Продолжать недоделанную очередь при запуске приложения.
    pub auto_resume_analysis: bool,
    pub language: String,
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
}
