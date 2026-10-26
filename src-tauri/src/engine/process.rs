use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::domain::error::{AppError, AppResult};

const READ_TIMEOUT: Duration = Duration::from_secs(120);

pub struct EngineProcess {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    name: String,
}

impl EngineProcess {
    pub async fn spawn(path: &Path) -> AppResult<Self> {
        if !path.is_file() {
            return Err(AppError::Engine(format!(
                "Исполняемый файл Stockfish не найден: {}",
                path.display()
            )));
        }
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|err| AppError::Engine(format!("Не удалось запустить Stockfish: {err}")))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::Engine("Не удалось открыть stdin Stockfish".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::Engine("Не удалось открыть stdout Stockfish".to_string()))?;
        let mut process = EngineProcess {
            child,
            stdin,
            reader: BufReader::new(stdout),
            name: "Stockfish".to_string(),
        };
        process.send("uci").await?;
        let mut name = None;
        loop {
            let line = process.read_line().await?;
            if let Some(value) = line.strip_prefix("id name ") {
                name = Some(value.trim().to_string());
            }
            if line.trim() == "uciok" {
                break;
            }
        }
        process.name = name.unwrap_or_else(|| "Stockfish".to_string());
        let threads = std::thread::available_parallelism()
            .map(|value| value.get().clamp(1, 4))
            .unwrap_or(2);
        process.set_option("Threads", &threads.to_string()).await?;
        process.set_option("Hash", "128").await?;
        process.send("isready").await?;
        loop {
            let line = process.read_line().await?;
            if line.trim() == "readyok" {
                break;
            }
        }
        Ok(process)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn send(&mut self, command: &str) -> AppResult<()> {
        self.stdin
            .write_all(command.as_bytes())
            .await
            .map_err(|err| AppError::Engine(format!("Ошибка записи в Stockfish: {err}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|err| AppError::Engine(format!("Ошибка записи в Stockfish: {err}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|err| AppError::Engine(format!("Ошибка flush Stockfish: {err}")))
    }

    pub async fn read_line(&mut self) -> AppResult<String> {
        let mut line = String::new();
        match timeout(READ_TIMEOUT, self.reader.read_line(&mut line)).await {
            Ok(Ok(0)) => Err(AppError::Engine(
                "Stockfish закрыл поток вывода".to_string(),
            )),
            Ok(Ok(_)) => Ok(line.trim_end().to_string()),
            Ok(Err(err)) => Err(AppError::Engine(format!("Ошибка чтения Stockfish: {err}"))),
            Err(_) => Err(AppError::Engine("Таймаут ответа Stockfish".to_string())),
        }
    }

    pub async fn set_option(&mut self, name: &str, value: &str) -> AppResult<()> {
        self.send(&format!("setoption name {name} value {value}"))
            .await
    }

    pub async fn set_multipv(&mut self, multipv: u32) -> AppResult<()> {
        self.set_option("MultiPV", &multipv.to_string()).await
    }

    pub async fn set_position(&mut self, fen: &str) -> AppResult<()> {
        self.send(&format!("position fen {fen}")).await
    }

    pub async fn go_depth(&mut self, depth: u32) -> AppResult<()> {
        self.send(&format!("go depth {depth}")).await
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        self.send("stop").await
    }

    pub async fn new_game(&mut self) -> AppResult<()> {
        self.send("ucinewgame").await?;
        self.send("isready").await?;
        loop {
            let line = self.read_line().await?;
            if line.trim() == "readyok" {
                return Ok(());
            }
        }
    }

    /// Аккуратно завершает процесс, при необходимости добивая его.
    pub async fn shutdown(&mut self) {
        let _ = self.send("quit").await;
        let _ = timeout(Duration::from_secs(2), self.child.wait()).await;
        let _ = self.child.start_kill();
    }
}

impl Drop for EngineProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
