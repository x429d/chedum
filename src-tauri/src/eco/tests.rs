use shakmaty::{san::SanPlus, Chess, Position};

use crate::domain::position::position_fen;
use crate::domain::stats::OpeningEntry;

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
