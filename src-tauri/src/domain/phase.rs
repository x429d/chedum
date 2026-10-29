use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Opening,
    Middlegame,
    Endgame,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Opening => "opening",
            Phase::Middlegame => "middlegame",
            Phase::Endgame => "endgame",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<Phase> {
        match value {
            "opening" => Some(Phase::Opening),
            "middlegame" => Some(Phase::Middlegame),
            "endgame" => Some(Phase::Endgame),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeFlag {
    Normal,
    TimeTrouble,
    ImpulsiveError,
    Overthink,
}

impl TimeFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeFlag::Normal => "normal",
            TimeFlag::TimeTrouble => "time_trouble",
            TimeFlag::ImpulsiveError => "impulsive_error",
            TimeFlag::Overthink => "overthink",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<TimeFlag> {
        match value {
            "normal" => Some(TimeFlag::Normal),
            "time_trouble" => Some(TimeFlag::TimeTrouble),
            "impulsive_error" => Some(TimeFlag::ImpulsiveError),
            "overthink" => Some(TimeFlag::Overthink),
            _ => None,
        }
    }
}
