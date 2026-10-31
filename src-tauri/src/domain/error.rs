use serde::ser::{Serialize, SerializeStruct, Serializer};

/// Single application error type. It is serialized to the frontend as
/// `{ code, message }`: the code is stable and localizable, the message is a
/// technical detail for diagnostics.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Engine error: {0}")]
    Engine(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),
    #[error("Engine busy: {0}")]
    Busy(String),
    #[error("Cancelled: {0}")]
    Cancelled(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Stable machine-readable code consumed by the frontend.
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
            AppError::Cancelled(_) => "cancelled",
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
            AppError::Network("Request timed out".to_string())
        } else {
            AppError::Network(value.to_string())
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
