use serde::{Deserialize, Serialize};

pub use crate::domain::enums::{Color, GameSource, TimeClass, UserColor};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub external_id: String,
    pub source: GameSource,
    pub pgn: String,
    pub initial_fen: Option<String>,
    pub white_username: String,
    pub black_username: String,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,
    pub user_color: UserColor,
    pub result: String,
    pub termination: Option<String>,
    pub time_control: Option<String>,
    pub time_class: Option<TimeClass>,
    pub eco: Option<String>,
    pub opening_name: Option<String>,
    pub opening_ply: Option<i32>,
    pub opening_source: Option<String>,
    pub played_at: i64,
    pub total_plies: i32,
    pub notes: String,
    pub is_favorite: bool,
    pub analysis_attempts: u32,
    pub last_analysis_error: Option<String>,
    /// Account the game came from; may be NULL for manual PGNs.
    pub account_id: Option<i64>,
}

/// Game before insertion into the database (SQLite assigns the id).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDraft {
    pub external_id: String,
    pub source: GameSource,
    pub pgn: String,
    pub initial_fen: Option<String>,
    pub white_username: String,
    pub black_username: String,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,
    pub user_color: UserColor,
    pub result: String,
    pub termination: Option<String>,
    pub time_control: Option<String>,
    pub time_class: Option<TimeClass>,
    pub eco: Option<String>,
    pub opening_name: Option<String>,
    #[serde(default)]
    pub opening_ply: Option<i32>,
    #[serde(default)]
    pub opening_source: Option<String>,
    pub played_at: i64,
    pub total_plies: i32,
    pub notes: String,
    pub is_favorite: bool,
    /// Account the game is linked to on import.
    #[serde(default)]
    pub account_id: Option<i64>,
}

impl Game {
    pub fn user_result(&self) -> Option<&'static str> {
        match self.user_color {
            UserColor::None => None,
            UserColor::White => match self.result.as_str() {
                "1-0" => Some("win"),
                "0-1" => Some("loss"),
                "1/2-1/2" => Some("draw"),
                _ => None,
            },
            UserColor::Black => match self.result.as_str() {
                "1-0" => Some("loss"),
                "0-1" => Some("win"),
                "1/2-1/2" => Some("draw"),
                _ => None,
            },
        }
    }
}

impl GameDraft {
    pub fn into_game(self, id: i64) -> Game {
        Game {
            id,
            external_id: self.external_id,
            source: self.source,
            pgn: self.pgn,
            initial_fen: self.initial_fen,
            white_username: self.white_username,
            black_username: self.black_username,
            white_elo: self.white_elo,
            black_elo: self.black_elo,
            user_color: self.user_color,
            result: self.result,
            termination: self.termination,
            time_control: self.time_control,
            time_class: self.time_class,
            eco: self.eco,
            opening_name: self.opening_name,
            opening_ply: self.opening_ply,
            opening_source: self.opening_source,
            played_at: self.played_at,
            total_plies: self.total_plies,
            notes: self.notes,
            is_favorite: self.is_favorite,
            analysis_attempts: 0,
            last_analysis_error: None,
            account_id: self.account_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameListItem {
    pub id: i64,
    pub source: GameSource,
    pub account_id: Option<i64>,
    pub account_username: Option<String>,
    pub account_platform: Option<crate::domain::account::AccountPlatform>,
    pub white_username: String,
    pub black_username: String,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,
    pub user_color: UserColor,
    pub result: String,
    pub time_class: Option<TimeClass>,
    pub time_control: Option<String>,
    pub eco: Option<String>,
    pub opening_name: Option<String>,
    pub played_at: i64,
    pub total_plies: i32,
    pub is_favorite: bool,
    pub notes: String,
    pub has_analysis: bool,
    pub user_accuracy: Option<f64>,
    pub user_acpl: Option<f64>,
    pub key_moments_count: Option<i32>,
    /// Origin of the accuracy value for the library tooltip.
    pub engine_name: Option<String>,
    pub analysis_depth: Option<i32>,
    pub accuracy_method: Option<String>,
    /// How many times game analysis failed; after 3 the game leaves the queue.
    pub analysis_attempts: u32,
    pub last_analysis_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameFilter {
    pub search: Option<String>,
    /// Show games of a single account; NULL — all accounts and manual PGNs.
    pub account_id: Option<i64>,
    pub color: Option<UserColor>,
    pub result: Option<String>,
    pub time_class: Option<TimeClass>,
    pub eco: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub analyzed_only: bool,
    pub favorites_only: bool,
    pub has_notes: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedGames {
    pub games: Vec<GameListItem>,
    pub total: u32,
    /// How many games in the selection already have an analysis (over the whole
    /// selection, not the page).
    pub analyzed: u32,
    /// How many games in the selection are marked as favorites.
    pub favorites: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAnalysis {
    pub ids: Vec<i64>,
    /// Total number of positions in the queue — the basis for the time estimate.
    pub positions: u64,
    /// Games excluded after three failed attempts.
    pub excluded: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub imported: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetail {
    pub game: Game,
    pub analysis: Option<crate::domain::analysis::GameAnalysis>,
    pub moves: Vec<crate::domain::analysis::MoveEvaluation>,
}
