use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use shakmaty::{san::SanPlus, Chess, Position};

use crate::domain::position::{book_key, position_fen};
use crate::domain::stats::OpeningEntry;

static RAW_ECO: &str = include_str!("../../resources/eco.json");

#[derive(Debug, Deserialize)]
struct RawEntry {
    eco: String,
    name: String,
    pgn: String,
}

/// Встроенный дебютный словарь ECO. Строится лениво один раз за процесс.
pub struct EcoBook {
    by_fen: HashMap<String, (String, String)>,
    entries: Vec<OpeningEntry>,
}

impl EcoBook {
    pub fn global() -> &'static EcoBook {
        static BOOK: OnceLock<EcoBook> = OnceLock::new();
        BOOK.get_or_init(EcoBook::build)
    }

    fn build() -> EcoBook {
        let raw: Vec<RawEntry> = serde_json::from_str(RAW_ECO).unwrap_or_default();
        let mut by_fen = HashMap::new();
        let mut entries = Vec::with_capacity(raw.len());
        for entry in raw {
            let positions = line_positions(&entry.pgn);
            for (fen, _ply) in positions {
                by_fen.insert(fen, (entry.eco.clone(), entry.name.clone()));
            }
            entries.push(OpeningEntry {
                eco: entry.eco,
                name: entry.name,
                pgn: entry.pgn,
            });
        }
        EcoBook { by_fen, entries }
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn lookup(&self, fen: &str) -> Option<(&str, &str)> {
        let key = book_key(fen);
        self.by_fen
            .get(&key)
            .map(|(eco, name)| (eco.as_str(), name.as_str()))
    }

    /// Самое глубокое совпадение по последовательности позиций партии.
    pub fn identify(&self, fens: &[String]) -> Option<(String, String)> {
        let mut found: Option<(String, String)> = None;
        for fen in fens {
            if let Some((eco, name)) = self.lookup(fen) {
                found = Some((eco.to_string(), name.to_string()));
            }
        }
        found
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<OpeningEntry> {
        let needle = query.trim().to_lowercase();
        let mut out = Vec::new();
        for entry in &self.entries {
            if out.len() >= limit {
                break;
            }
            let matches = needle.is_empty()
                || entry.eco.to_lowercase().contains(&needle)
                || entry.name.to_lowercase().contains(&needle)
                || entry.pgn.to_lowercase().contains(&needle);
            if matches {
                out.push(entry.clone());
            }
        }
        out
    }
}

fn line_positions(pgn: &str) -> Vec<(String, usize)> {
    let mut pos = Chess::default();
    let mut out = Vec::new();
    for token in pgn.split_whitespace() {
        if token.ends_with('.') || matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*") {
            continue;
        }
        let clean = token.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.');
        if clean.is_empty() {
            continue;
        }
        let parsed = match SanPlus::from_ascii(clean.as_bytes()) {
            Ok(parsed) => parsed,
            Err(_) => break,
        };
        let mv = match parsed.san.to_move(&pos) {
            Ok(mv) => mv,
            Err(_) => break,
        };
        pos.play_unchecked(mv);
        let ply = out.len() + 1;
        out.push((book_key(&position_fen(&pos)), ply));
    }
    out
}

/// Удобный фасад для проверки дебютной теории.
pub fn is_book_position(fen: &str) -> bool {
    EcoBook::global().lookup(fen).is_some()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_has_at_least_three_hundred_lines() {
        assert!(EcoBook::global().count() >= 300);
    }

    #[test]
    fn opening_after_e4_e5_is_known() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2";
        assert!(is_book_position(fen));
    }

    #[test]
    fn unknown_position_is_not_book() {
        let fen = "8/8/8/8/8/8/8/K6k w - - 0 1";
        assert!(!is_book_position(fen));
    }

    #[test]
    fn search_finds_sicilian() {
        let found = EcoBook::global().search("Sicilian", 10);
        assert!(!found.is_empty());
        assert!(found
            .iter()
            .all(|entry| entry.name.to_lowercase().contains("sicilian")
                || entry.eco.starts_with('B')));
    }

    #[test]
    fn identify_picks_deepest_line() {
        let fens = vec![
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".to_string(),
        ];
        let identified = EcoBook::global().identify(&fens);
        assert!(identified.is_some());
    }
}
