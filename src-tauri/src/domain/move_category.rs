use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveCategory {
    Book,
    Forced,
    Brilliant,
    Great,
    Best,
    Excellent,
    Good,
    Miss,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveCategory {
    pub const ALL: [MoveCategory; 11] = [
        MoveCategory::Brilliant,
        MoveCategory::Great,
        MoveCategory::Best,
        MoveCategory::Excellent,
        MoveCategory::Good,
        MoveCategory::Book,
        MoveCategory::Inaccuracy,
        MoveCategory::Mistake,
        MoveCategory::Miss,
        MoveCategory::Blunder,
        MoveCategory::Forced,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            MoveCategory::Book => "book",
            MoveCategory::Forced => "forced",
            MoveCategory::Brilliant => "brilliant",
            MoveCategory::Great => "great",
            MoveCategory::Best => "best",
            MoveCategory::Excellent => "excellent",
            MoveCategory::Good => "good",
            MoveCategory::Miss => "miss",
            MoveCategory::Inaccuracy => "inaccuracy",
            MoveCategory::Mistake => "mistake",
            MoveCategory::Blunder => "blunder",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<MoveCategory> {
        MoveCategory::ALL
            .iter()
            .copied()
            .find(|category| category.as_str() == value)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            MoveCategory::Book => "📖",
            MoveCategory::Forced => "□",
            MoveCategory::Brilliant => "!!",
            MoveCategory::Great => "!",
            MoveCategory::Best => "★",
            MoveCategory::Excellent => "👍",
            MoveCategory::Good => "✓",
            MoveCategory::Miss => "✕",
            MoveCategory::Inaccuracy => "?!",
            MoveCategory::Mistake => "?",
            MoveCategory::Blunder => "??",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(
            self,
            MoveCategory::Inaccuracy
                | MoveCategory::Mistake
                | MoveCategory::Miss
                | MoveCategory::Blunder
        )
    }

    pub fn is_mistake_or_blunder(self) -> bool {
        matches!(self, MoveCategory::Mistake | MoveCategory::Blunder)
    }

    pub fn is_key_moment(self) -> bool {
        matches!(
            self,
            MoveCategory::Brilliant
                | MoveCategory::Great
                | MoveCategory::Miss
                | MoveCategory::Mistake
                | MoveCategory::Blunder
        )
    }

    pub fn needs_coach(self) -> bool {
        matches!(
            self,
            MoveCategory::Brilliant
                | MoveCategory::Great
                | MoveCategory::Inaccuracy
                | MoveCategory::Mistake
                | MoveCategory::Miss
                | MoveCategory::Blunder
        )
    }
}
