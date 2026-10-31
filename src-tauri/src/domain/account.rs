use serde::{Deserialize, Serialize};

use crate::domain::error::{AppError, AppResult};

/// Площадка, с которой приходят партии.
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
    /// Сколько партий привязано к аккаунту.
    pub games: u32,
    /// Сколько из них уже разобрано.
    pub analyzed: u32,
}

/// Приводит ник к безопасному виду и проверяет, что он вообще может
/// существовать на выбранной площадке.
pub fn normalize_username(platform: AccountPlatform, value: &str) -> AppResult<String> {
    let username = value.trim();
    if username.is_empty() {
        return Err(AppError::Validation("Укажите ник аккаунта".to_string()));
    }
    if username.chars().count() > 50 {
        return Err(AppError::Validation(
            "Ник не должен быть длиннее 50 символов".to_string(),
        ));
    }
    let valid = username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if !valid {
        return Err(AppError::Validation(
            "Ник может содержать только латинские буквы, цифры, «_» и «-»".to_string(),
        ));
    }
    if platform == AccountPlatform::Lichess && username.chars().count() < 2 {
        return Err(AppError::Validation(
            "Ник Lichess должен быть не короче 2 символов".to_string(),
        ));
    }
    Ok(username.to_string())
}
