use std::cmp::Reverse;
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

/// Версия встроенного датасета: FNV-1a от содержимого. Меняется вместе с файлом,
/// поэтому пересчёт дебютов запускается ровно один раз на новую базу.
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

/// Приводит имя из PGN-заголовка к читаемому виду: схлопывает пробелы,
/// убирает хвостовую пунктуацию и повторяющиеся сегменты.
pub fn normalize_opening_name(value: &str) -> Option<String> {
    let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = cleaned.trim_matches(|ch: char| ch == '.' || ch == ',' || ch.is_whitespace());
    if trimmed.is_empty() {
        return None;
    }
    let mut segments: Vec<&str> = Vec::new();
    for segment in trimmed.split(", ") {
        if segments.last() != Some(&segment) {
            segments.push(segment);
        }
    }
    Some(segments.join(", "))
}

/// Имя из slug-URL Chess.com: `.../openings/Vienna-Game` → `Vienna Game`.
pub fn name_from_eco_url(url: &str) -> Option<String> {
    let slug = url.trim().trim_end_matches('/').rsplit('/').next()?;
    if slug.is_empty() {
        return None;
    }
    normalize_opening_name(&slug.replace(['-', '_'], " "))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn fens_from_san(line: &[&str]) -> Vec<String> {
        let mut pos = Chess::default();
        let mut out = Vec::new();
        for san in line {
            let parsed = SanPlus::from_ascii(san.as_bytes()).expect("san");
            let mv = parsed.san.to_move(&pos).expect("legal move");
            pos.play_unchecked(mv);
            out.push(position_fen(&pos));
        }
        out
    }

    fn dataset_entry(pgn: &str) -> OpeningEntry {
        EcoBook::global()
            .entries()
            .iter()
            .find(|entry| entry.pgn == pgn)
            .expect("dataset entry")
            .clone()
    }

    fn identify_line(line: &[&str]) -> OpeningMatch {
        EcoBook::global()
            .identify_with_ply(&fens_from_san(line))
            .expect("opening")
    }

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
        let fens = fens_from_san(&["e4", "e5"]);
        let found = EcoBook::global().identify_with_ply(&fens).unwrap();
        assert_eq!(found.ply, 2);
        assert_eq!(found.eco, "C20");
    }

    #[test]
    fn first_move_uses_exact_dataset_name() {
        let entry = dataset_entry("1. e4");
        assert_eq!(identify_line(&["e4"]), OpeningMatch {
            eco: entry.eco,
            name: entry.name,
            ply: 1,
        });
    }

    #[test]
    fn vienna_game_is_not_falkbeer() {
        let entry = dataset_entry("1. e4 e5 2. Nc3");
        let found = identify_line(&["e4", "e5", "Nc3"]);
        assert_eq!(found.eco, entry.eco);
        assert_eq!(found.name, entry.name);
        assert!(found.eco.starts_with("C2"), "{}", found.eco);
        assert!(found.name.contains("Vienna Game"), "{}", found.name);
        assert!(!found.name.contains("Falkbeer"), "{}", found.name);
    }

    #[test]
    fn ruy_lopez_family_is_c60_to_c99() {
        let found = identify_line(&["e4", "e5", "Nf3", "Nc6", "Bb5"]);
        assert!(found.name.contains("Ruy Lopez"), "{}", found.name);
        let number: u32 = found.eco[1..].parse().unwrap();
        assert!((60..=99).contains(&number), "{}", found.eco);
    }

    #[test]
    fn scandinavian_is_b01() {
        let found = identify_line(&["e4", "d5"]);
        assert_eq!(found.eco, "B01");
        assert!(found.name.contains("Scandinavian"), "{}", found.name);
    }

    #[test]
    fn queens_gambit_family_is_d() {
        let found = identify_line(&["d4", "d5", "c4"]);
        assert!(found.eco.starts_with('D'), "{}", found.eco);
        assert!(found.name.contains("Queen's Gambit"), "{}", found.name);
    }

    #[test]
    fn transposition_gives_the_same_opening() {
        let left = identify_line(&["d4", "d5", "c4", "e6", "Nc3", "Nf6"]);
        let right = identify_line(&["d4", "Nf6", "c4", "e6", "Nc3", "d5"]);
        assert_eq!(left, right);
    }

    #[test]
    fn identified_names_are_exact_dataset_entries() {
        let lines: [&[&str]; 5] = [
            &["e4", "e5", "Nc3"],
            &["e4", "e5", "Nf3", "Nc6", "Bb5"],
            &["e4", "d5"],
            &["d4", "d5", "c4"],
            &["e4", "e5", "Nf3", "Nc6", "Bc4"],
        ];
        for line in lines {
            let found = identify_line(line);
            assert!(
                EcoBook::global()
                    .entries()
                    .iter()
                    .any(|entry| entry.eco == found.eco && entry.name == found.name),
                "{} {}",
                found.eco,
                found.name
            );
        }
    }

    #[test]
    fn header_names_are_normalized_not_glued() {
        assert_eq!(
            normalize_opening_name("  Vienna   Game. ").as_deref(),
            Some("Vienna Game")
        );
        assert_eq!(
            normalize_opening_name("Sicilian Defense, Sicilian Defense").as_deref(),
            Some("Sicilian Defense")
        );
        assert_eq!(normalize_opening_name("   "), None);
    }

    #[test]
    fn eco_url_slug_becomes_a_name() {
        assert_eq!(
            name_from_eco_url("https://www.chess.com/openings/Vienna-Game").as_deref(),
            Some("Vienna Game")
        );
    }
}
