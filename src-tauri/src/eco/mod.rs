mod identify;
mod line;

pub use crate::eco::identify::{name_from_eco_url, normalize_opening_name};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use crate::domain::position::book_key;
use crate::domain::stats::OpeningEntry;
use crate::eco::line::line_positions;

static RAW_ECO: &str = include_str!("../../resources/eco.json");

#[derive(Debug, Deserialize)]
struct RawEntry {
    eco: String,
    name: String,
    pgn: String,
}

/// Самое раннее точное описание позиции: минимальный остаток строки,
/// затем максимальная глубина, затем устойчивый лексикографический порядок.
#[derive(Debug, Clone)]
struct Candidate {
    eco: String,
    name: String,
    remaining: usize,
    ply: usize,
}

impl Candidate {
    fn better_than(&self, other: &Candidate) -> bool {
        (self.remaining, Reverse(self.ply), &self.eco, &self.name)
            < (
                other.remaining,
                Reverse(other.ply),
                &other.eco,
                &other.name,
            )
    }
}

/// Совпадение позиции с базой дебютов.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningMatch {
    pub eco: String,
    pub name: String,
    /// Полуход, на котором сделан последний ход найденной линии.
    pub ply: usize,
}

/// Встроенный дебютный словарь ECO. Строится лениво один раз за процесс.
pub struct EcoBook {
    by_fen: HashMap<String, Candidate>,
    entries: Vec<OpeningEntry>,
}
impl EcoBook {
    pub fn global() -> &'static EcoBook {
        static BOOK: OnceLock<EcoBook> = OnceLock::new();
        BOOK.get_or_init(EcoBook::build)
    }

    fn build() -> EcoBook {
        let raw: Vec<RawEntry> = serde_json::from_str(RAW_ECO).unwrap_or_default();
        let mut by_fen: HashMap<String, Candidate> = HashMap::new();
        let mut entries = Vec::with_capacity(raw.len());
        for entry in raw {
            let positions = line_positions(&entry.pgn);
            let total = positions.len();
            for (fen, ply) in positions {
                let candidate = Candidate {
                    eco: entry.eco.clone(),
                    name: entry.name.clone(),
                    remaining: total.saturating_sub(ply),
                    ply,
                };
                match by_fen.get(&fen) {
                    Some(current) if !candidate.better_than(current) => {}
                    _ => {
                        by_fen.insert(fen, candidate);
                    }
                }
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

    pub fn entries(&self) -> &[OpeningEntry] {
        &self.entries
    }

    pub fn lookup(&self, fen: &str) -> Option<(&str, &str)> {
        let key = book_key(fen);
        self.by_fen
            .get(&key)
            .map(|candidate| (candidate.eco.as_str(), candidate.name.as_str()))
    }

    /// Самое глубокое совпадение по последовательности позиций партии.
    pub fn identify(&self, fens: &[String]) -> Option<(String, String)> {
        self.identify_with_ply(fens)
            .map(|found| (found.eco, found.name))
    }

    /// То же, что `identify`, но с полуходом найденной линии.
    pub fn identify_with_ply(&self, fens: &[String]) -> Option<OpeningMatch> {
        let mut found: Option<OpeningMatch> = None;
        for (index, fen) in fens.iter().enumerate() {
            let key = book_key(fen);
            if let Some(candidate) = self.by_fen.get(&key) {
                found = Some(OpeningMatch {
                    eco: candidate.eco.clone(),
                    name: candidate.name.clone(),
                    ply: index + 1,
                });
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
pub fn dataset_version() -> String {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION
        .get_or_init(|| {
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for byte in RAW_ECO.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            format!("chess-openings-{hash:016x}")
        })
        .clone()
}

/// Удобный фасад для проверки дебютной теории.
pub fn is_book_position(fen: &str) -> bool {
    EcoBook::global().lookup(fen).is_some()
}
