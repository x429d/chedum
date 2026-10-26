use super::*;
use crate::domain::game::{GameSource, UserColor};

const LEGAL: &str = include_str!("../../resources/demo_legal.pgn");

#[test]
fn parses_demo_game_with_clocks() {
    let game = parse_single(LEGAL).unwrap();
    assert_eq!(game.moves.len(), 13);
    assert_eq!(game.result, "1-0");
    assert_eq!(game.moves[0].san, "e4");
    assert_eq!(game.moves[0].uci, "e2e4");
    assert!(game.moves[0].clock_after.unwrap() > 0.0);
    assert_eq!(game.moves[1].san, "e5");
    assert_eq!(game.moves[12].san, "Nd5#");
}

#[test]
fn builds_game_draft_with_user_color() {
    let parsed = parse_single(LEGAL).unwrap();
    let draft = build_draft(
        &parsed,
        GameSource::Manual,
        "test-1".to_string(),
        "CHENAL_PLAYER",
        None,
        None,
    );
    assert_eq!(draft.user_color, UserColor::Black);
    assert_eq!(draft.white_username, "pawn_storm");
    assert_eq!(draft.black_username, "chenal_player");
    assert_eq!(draft.total_plies, 13);
    assert_eq!(draft.eco.as_deref(), Some("C41"));
    assert_eq!(
        draft.time_class,
        Some(crate::domain::game::TimeClass::Blitz)
    );
    assert!(draft.played_at > 0);
}

#[test]
fn parses_multiple_games() {
    let combined = format!("{LEGAL}\n{LEGAL}");
    let report = parse_pgn(&combined);
    assert_eq!(report.games.len(), 2);
    assert!(report.errors.is_empty());
}

#[test]
fn roundtrips_to_pgn() {
    let parsed = parse_single(LEGAL).unwrap();
    let pgn = parsed.to_pgn();
    assert!(pgn.contains("[White \"pawn_storm\"]"));
    assert!(pgn.contains("1. e4"));
    assert!(pgn.contains("[%clk"));
    let reparsed = parse_single(&pgn).unwrap();
    assert_eq!(reparsed.moves.len(), 13);
    assert_eq!(reparsed.result, "1-0");
}

#[test]
fn all_demo_games_are_legal() {
    let files = [
        include_str!("../../resources/demo_opera.pgn"),
        include_str!("../../resources/demo_immortal.pgn"),
        include_str!("../../resources/demo_evergreen.pgn"),
        LEGAL,
    ];
    for pgn in files {
        let report = parse_pgn(pgn);
        assert!(report.errors.is_empty(), "PGN errors: {:?}", report.errors);
        assert_eq!(report.games.len(), 1);
    }
}

#[test]
fn supports_custom_start_position() {
    let pgn = "[FEN \"8/8/8/8/8/8/8/K6k w - - 0 1\"]\n[SetUp \"1\"]\n\n1. Kb2 Kg1 2. Kc3 Kf1 *";
    let game = parse_single(pgn).unwrap();
    assert_eq!(game.moves.len(), 4);
    assert!(game.initial_fen.is_some());
    let draft = build_draft(
        &game,
        GameSource::Manual,
        "custom".to_string(),
        "",
        None,
        None,
    );
    assert_eq!(draft.user_color, UserColor::None);
}

#[test]
fn reports_invalid_game_without_panicking() {
    let pgn = "[White \"A\"]\n[Black \"B\"]\n\n1. e4 e5 2. Ke3 *";
    let report = parse_pgn(pgn);
    assert!(report.games.is_empty());
    assert!(!report.errors.is_empty());
}
