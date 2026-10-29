use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Color::White => "white",
            Color::Black => "black",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<Color> {
        match value {
            "white" => Some(Color::White),
            "black" => Some(Color::Black),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserColor {
    White,
    Black,
    None,
}

impl UserColor {
    pub fn as_str(self) -> &'static str {
        match self {
            UserColor::White => "white",
            UserColor::Black => "black",
            UserColor::None => "none",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<UserColor> {
        match value {
            "white" => Some(UserColor::White),
            "black" => Some(UserColor::Black),
            "none" => Some(UserColor::None),
            _ => None,
        }
    }

    pub fn color(self) -> Option<Color> {
        match self {
            UserColor::White => Some(Color::White),
            UserColor::Black => Some(Color::Black),
            UserColor::None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameSource {
    Chesscom,
    Manual,
}

impl GameSource {
    pub fn as_str(self) -> &'static str {
        match self {
            GameSource::Chesscom => "chesscom",
            GameSource::Manual => "manual",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<GameSource> {
        match value {
            "chesscom" => Some(GameSource::Chesscom),
            "manual" => Some(GameSource::Manual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeClass {
    Bullet,
    Blitz,
    Rapid,
    Classical,
}

impl TimeClass {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeClass::Bullet => "bullet",
            TimeClass::Blitz => "blitz",
            TimeClass::Rapid => "rapid",
            TimeClass::Classical => "classical",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<TimeClass> {
        match value {
            "bullet" => Some(TimeClass::Bullet),
            "blitz" => Some(TimeClass::Blitz),
            "rapid" => Some(TimeClass::Rapid),
            "classical" | "daily" => Some(TimeClass::Classical),
            _ => None,
        }
    }
}

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
}

/// Партия до вставки в базу (id присваивает SQLite).
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
    /// Превращает черновик в сущность с присвоенным идентификатором.
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameListItem {
    pub id: i64,
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
    /// Происхождение точности для подсказки в библиотеке.
    pub engine_name: Option<String>,
    pub analysis_depth: Option<i32>,
    pub accuracy_method: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameFilter {
    pub search: Option<String>,
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
    /// Сколько партий выборки уже имеют анализ (по всей выборке, не по странице).
    pub analyzed: u32,
    /// Сколько партий выборки отмечено избранными.
    pub favorites: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAnalysis {
    pub ids: Vec<i64>,
    /// Суммарное число позиций очереди — база для оценки времени.
    pub positions: u64,
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
