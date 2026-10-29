use serde::ser::{Serialize, SerializeStruct, Serializer};

/// Единый тип ошибки приложения. Сериализуется во фронтенд как `{ code, message }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Ошибка базы данных: {0}")]
    Database(String),
    #[error("Ошибка ввода-вывода: {0}")]
    Io(String),
    #[error("Сетевая ошибка: {0}")]
    Network(String),
    #[error("Ошибка движка: {0}")]
    Engine(String),
    #[error("Ошибка разбора: {0}")]
    Parse(String),
    #[error("Объект не найден: {0}")]
    NotFound(String),
    #[error("Некорректные данные: {0}")]
    Validation(String),
    #[error("Превышен лимит запросов: {0}")]
    RateLimited(String),
    #[error("Движок занят: {0}")]
    Busy(String),
    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

impl AppError {
    /// Стабильный машинный код ошибки для фронтенда.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database",
            AppError::Io(_) => "io",
            AppError::Network(_) => "network",
            AppError::Engine(_) => "engine",
            AppError::Parse(_) => "parse",
            AppError::NotFound(_) => "not_found",
            AppError::Validation(_) => "validation",
            AppError::RateLimited(_) => "rate_limited",
            AppError::Busy(_) => "busy",
            AppError::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        AppError::Database(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Parse(format!("JSON: {value}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            AppError::Network("Таймаут запроса к Chess.com".to_string())
        } else {
            AppError::Network(value.to_string())
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
