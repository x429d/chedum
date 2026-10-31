use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

use crate::domain::error::{AppError, AppResult};

const USER_AGENT: &str = "Chenal/0.1.0 (macOS; native chess analytics studio)";
const MIN_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveList {
    #[serde(default)]
    pub archives: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RemotePlayer {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub rating: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RemoteGame {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub pgn: String,
    #[serde(default)]
    pub time_control: String,
    #[serde(default)]
    pub end_time: i64,
    #[serde(default)]
    pub time_class: String,
    #[serde(default)]
    pub rules: String,
    #[serde(default)]
    pub white: RemotePlayer,
    #[serde(default)]
    pub black: RemotePlayer,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchiveGames {
    #[serde(default)]
    pub games: Vec<RemoteGame>,
}

#[async_trait]
pub trait ChessComApi: Send + Sync {
    async fn archives(&self, username: &str) -> AppResult<Vec<String>>;
    async fn archive_games(&self, archive_url: &str) -> AppResult<Vec<RemoteGame>>;
}

pub struct ChessComClient {
    http: Client,
    base: String,
    last_request: Mutex<Option<Instant>>,
}

impl ChessComClient {
    pub fn new() -> AppResult<Self> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|err| AppError::Network(format!("Could not create the HTTP client: {err}")))?;
        Ok(ChessComClient {
            http,
            base: "https://api.chess.com/pub".to_string(),
            last_request: Mutex::new(None),
        })
    }

    async fn throttle(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(previous) = *last {
            let elapsed = previous.elapsed();
            if elapsed < MIN_INTERVAL {
                sleep(MIN_INTERVAL - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> AppResult<T> {
        let mut attempt = 0u32;
        loop {
            self.throttle().await;
            let response = self.http.get(url).send().await?;
            let status = response.status();
            if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                if attempt >= 2 {
                    return Err(AppError::RateLimited(format!(
                        "Chess.com returned {status} after three attempts"
                    )));
                }
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(2u64 << attempt);
                sleep(Duration::from_secs(retry_after.min(30))).await;
                attempt += 1;
                continue;
            }
            if !status.is_success() {
                return Err(AppError::Network(format!(
                    "Chess.com returned {status} for {url}"
                )));
            }
            return response.json::<T>().await.map_err(AppError::from);
        }
    }
}

#[async_trait]
impl ChessComApi for ChessComClient {
    async fn archives(&self, username: &str) -> AppResult<Vec<String>> {
        let username = username.trim();
        if username.is_empty()
            || !username
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(AppError::Validation(
                "Invalid Chess.com username".to_string(),
            ));
        }
        let url = format!("{}/player/{username}/games/archives", self.base);
        let list: ArchiveList = self.get_json(&url).await?;
        Ok(list.archives)
    }

    async fn archive_games(&self, archive_url: &str) -> AppResult<Vec<RemoteGame>> {
        let archive: ArchiveGames = self.get_json(archive_url).await?;
        Ok(archive.games)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn client_has_required_user_agent() {
        let client = ChessComClient::new().unwrap();
        assert_eq!(client.base, "https://api.chess.com/pub");
        assert_eq!(
            USER_AGENT,
            "Chenal/0.1.0 (macOS; native chess analytics studio)"
        );
    }

    #[test]
    fn deserializes_remote_game() {
        let json = r#"{
            "url": "https://www.chess.com/game/live/1",
            "pgn": "1. e4 e5 *",
            "time_control": "600+5",
            "end_time": 1700000000,
            "time_class": "rapid",
            "rules": "chess",
            "white": {"username": "Alice", "result": "win", "rating": 1800},
            "black": {"username": "Bob", "result": "checkmated", "rating": 1750}
        }"#;
        let game: RemoteGame = serde_json::from_str(json).unwrap();
        assert_eq!(game.white.username, "Alice");
        assert_eq!(game.black.rating, Some(1750));
        assert_eq!(game.time_class, "rapid");
    }
}
