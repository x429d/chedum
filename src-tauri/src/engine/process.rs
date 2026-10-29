use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;

use crate::domain::error::{AppError, AppResult};

/// Stockfish печатает прогресс не реже раза в секунду, поэтому долгая тишина
/// означает зависший процесс. Короткий таймаут не даёт очереди анализа
/// застрять на одной позиции на минуты.
const READ_TIMEOUT: Duration = Duration::from_secs(30);
/// Защита от бесконечной строки, если движок начнёт печатать мусор.
const MAX_LINE_BYTES: usize = 1024 * 1024;

pub struct EngineProcess {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    /// Байты, уже прочитанные из канала, но ещё не составившие строку.
    /// Хранятся в структуре, иначе таймаут выбрасывал бы половину строки
    /// и весь UCI-поток после него был бы сдвинут.
    pending: Vec<u8>,
    name: String,
}

impl EngineProcess {
    pub async fn spawn(path: &Path) -> AppResult<Self> {
        let threads = std::thread::available_parallelism()
            .map(|value| value.get().clamp(1, 4))
            .unwrap_or(2);
        EngineProcess::spawn_with(path, threads, 128).await
    }

    /// `threads` — потоков на процесс, `hash_mb` — размер хеш-таблицы.
    pub async fn spawn_with(path: &Path, threads: usize, hash_mb: u32) -> AppResult<Self> {
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
            pending: Vec::new(),
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
        process
            .set_option("Threads", &threads.clamp(1, 64).to_string())
            .await?;
        process
            .set_option("Hash", &hash_mb.clamp(16, 1024).to_string())
            .await?;
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

    /// Читает одну строку, сохраняя частично полученные данные между вызовами.
    pub async fn read_line(&mut self) -> AppResult<String> {
        loop {
            if let Some(index) = self.pending.iter().position(|byte| *byte == b'\n') {
                let mut line: Vec<u8> = self.pending.drain(..=index).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(String::from_utf8_lossy(&line).to_string());
            }
            if self.pending.len() > MAX_LINE_BYTES {
                self.pending.clear();
                return Err(AppError::Engine(
                    "Слишком длинная строка вывода Stockfish".to_string(),
                ));
            }
            let mut chunk: Vec<u8> = Vec::new();
            match timeout(READ_TIMEOUT, self.reader.fill_buf()).await {
                Ok(Ok([])) => {
                    return Err(AppError::Engine(
                        "Stockfish закрыл поток вывода".to_string(),
                    ))
                }
                Ok(Ok(buffer)) => chunk.extend_from_slice(buffer),
                Ok(Err(err)) => {
                    return Err(AppError::Engine(format!("Ошибка чтения Stockfish: {err}")))
                }
                Err(_) => return Err(AppError::Engine("Таймаут ответа Stockfish".to_string())),
            }
            self.reader.consume(chunk.len());
            self.pending.extend_from_slice(&chunk);
        }
    }

    /// Процесс ещё жив и готов принимать команды.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
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
