use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

use crate::domain::error::{AppError, AppResult};

const USER_AGENT: &str = concat!(
    "Chedum/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/x429d/chedum)"
);
const MIN_INTERVAL: Duration = Duration::from_secs(1);

#[async_trait]
pub trait LichessApi: Send + Sync {
    /// Returns the PGN of all games for a user; `since_ms` is the inclusive
    /// lower bound on game end time in Unix milliseconds.
    async fn user_games_pgn(&self, username: &str, since_ms: Option<i64>) -> AppResult<String>;
}

pub struct LichessClient {
    http: Client,
    base: String,
    last_request: Mutex<Option<Instant>>,
}

impl LichessClient {
    pub fn new() -> AppResult<Self> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(900))
            .build()
            .map_err(|err| AppError::Network(format!("Could not create the HTTP client: {err}")))?;
        Ok(LichessClient {
            http,
            base: "https://lichess.org".to_string(),
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

    fn games_url(&self, username: &str, since_ms: Option<i64>) -> String {
        let mut url = format!(
            "{}/api/games/user/{username}?clocks=true&opening=true&sort=dateAsc",
            self.base
        );
        if let Some(since) = since_ms {
            url.push_str(&format!("&since={since}"));
        }
        url
    }

    async fn get_pgn(&self, url: &str) -> AppResult<String> {
        let mut attempt = 0u32;
        loop {
            self.throttle().await;
            let response = self
                .http
                .get(url)
                .header("Accept", "application/x-chess-pgn")
                .send()
                .await?;
            let status = response.status();
            if status == StatusCode::NOT_FOUND {
                return Err(AppError::NotFound(
                    "Lichess profile not found. Check the username.".to_string(),
                ));
            }
            if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                if attempt >= 2 {
                    return Err(AppError::RateLimited(format!(
                        "Lichess returned {status} after three attempts"
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
                    "Lichess returned {status} for {url}"
                )));
            }
            return response.text().await.map_err(AppError::from);
        }
    }
}

#[async_trait]
impl LichessApi for LichessClient {
    async fn user_games_pgn(&self, username: &str, since_ms: Option<i64>) -> AppResult<String> {
        let username = username.trim();
        let url = self.games_url(username, since_ms);
        self.get_pgn(&url).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn builds_export_url_with_optional_since() {
        let client = LichessClient::new().unwrap();
        assert_eq!(
            client.games_url("alice", None),
            "https://lichess.org/api/games/user/alice?clocks=true&opening=true&sort=dateAsc"
        );
        assert_eq!(
            client.games_url("alice", Some(1_700_000_000_000)),
            "https://lichess.org/api/games/user/alice?clocks=true&opening=true&sort=dateAsc&since=1700000000000"
        );
    }
}
