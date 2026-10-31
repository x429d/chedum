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
    Lichess,
    Manual,
}

impl GameSource {
    pub fn as_str(self) -> &'static str {
        match self {
            GameSource::Chesscom => "chesscom",
            GameSource::Lichess => "lichess",
            GameSource::Manual => "manual",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<GameSource> {
        match value {
            "chesscom" => Some(GameSource::Chesscom),
            "lichess" => Some(GameSource::Lichess),
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
