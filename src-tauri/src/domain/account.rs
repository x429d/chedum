use serde::{Deserialize, Serialize};

use crate::domain::error::{AppError, AppResult};

/// Platform the games come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountPlatform {
    Chesscom,
    Lichess,
}

impl AccountPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            AccountPlatform::Chesscom => "chesscom",
            AccountPlatform::Lichess => "lichess",
        }
    }

    pub fn from_str_opt(value: &str) -> Option<AccountPlatform> {
        match value {
            "chesscom" => Some(AccountPlatform::Chesscom),
            "lichess" => Some(AccountPlatform::Lichess),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub platform: AccountPlatform,
    pub username: String,
    pub created_at: String,
    pub last_synced_at: Option<String>,
    /// Number of games linked to this account.
    pub games: u32,
    /// How many of them have already been analyzed.
    pub analyzed: u32,
}

/// Normalizes the username to a safe form and checks that it could exist on the
/// selected platform.
pub fn normalize_username(platform: AccountPlatform, value: &str) -> AppResult<String> {
    let username = value.trim();
    if username.is_empty() {
        return Err(AppError::Validation(
            "Enter an account username".to_string(),
        ));
    }
    if username.chars().count() > 50 {
        return Err(AppError::Validation(
            "Username must be at most 50 characters".to_string(),
        ));
    }
    let valid = username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if !valid {
        return Err(AppError::Validation(
            "Username may contain only Latin letters, digits, «_» and «-»".to_string(),
        ));
    }
    if platform == AccountPlatform::Lichess && username.chars().count() < 2 {
        return Err(AppError::Validation(
            "Lichess username must be at least 2 characters".to_string(),
        ));
    }
    Ok(username.to_string())
}
