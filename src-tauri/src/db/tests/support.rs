use crate::domain::analysis::{
    AnalysisDraft, CategoryCounts, GameAnalysis, MoveCategory, MoveEvaluation, Phase, PvLine,
    TimeFlag,
};
use crate::domain::game::{Color, GameDraft, GameSource, TimeClass, UserColor};

pub(super) const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub(super) fn draft(
    external_id: &str,
    white: &str,
    black: &str,
    user_color: UserColor,
    result: &str,
    time_class: TimeClass,
    played_at: i64,
) -> GameDraft {
    GameDraft {
        external_id: external_id.to_string(),
        source: GameSource::Manual,
        pgn: "1. e4 e5 *".to_string(),
        initial_fen: None,
        white_username: white.to_string(),
        black_username: black.to_string(),
        white_elo: Some(1800),
        black_elo: Some(1700),
        user_color,
        result: result.to_string(),
        termination: Some("Normal".to_string()),
        time_control: Some("600+0".to_string()),
        time_class: Some(time_class),
        eco: Some("C41".to_string()),
        opening_name: Some("Philidor Defense".to_string()),
        opening_ply: Some(3),
        opening_source: Some("position".to_string()),
        played_at,
        total_plies: 2,
        notes: String::new(),
        is_favorite: false,
        account_id: None,
    }
}

pub(super) fn move_eval(
    game_id: i64,
    ply: i32,
    phase: Phase,
    classification: MoveCategory,
    flag: TimeFlag,
) -> MoveEvaluation {
    MoveEvaluation {
        game_id,
        ply,
        player_color: if ply % 2 == 1 {
            Color::White
        } else {
            Color::Black
        },
        phase,
        fen_before: START.to_string(),
        fen_after: START.to_string(),
        played_uci: "e2e4".to_string(),
        played_san: "e4".to_string(),
        best_uci: Some("e2e4".to_string()),
        best_san: Some("e4".to_string()),
        eval_cp: Some(20),
        eval_mate: None,
        best_eval_cp: Some(25),
        best_eval_mate: None,
        win_prob_white: 55.0,
        win_prob_loss: 1.0,
        move_accuracy: 97.0,
        classification,
        is_critical: false,
        is_key_moment: false,
        material_diff_white: 0,
        clock_remaining_sec: Some(500.0),
        time_spent_sec: Some(5.0),
        time_flag: flag,
        explanation_tag: None,
        explanation_text: None,
        pv_lines: vec![PvLine {
            rank: 1,
            score_cp: Some(25),
            score_mate: None,
            win_prob: 55.0,
            uci_moves: vec!["e2e4".to_string()],
            san_moves: vec!["e4".to_string()],
        }],
        user_comment: String::new(),
    }
}

pub(super) fn analysis(
    game_id: i64,
    white_accuracy: f64,
    black_accuracy: f64,
    moves: Vec<MoveEvaluation>,
) -> AnalysisDraft {
    let mut counts = CategoryCounts::new();
    counts.increment(Color::White, MoveCategory::Best);
    counts.increment(Color::Black, MoveCategory::Inaccuracy);
    AnalysisDraft {
        analysis: GameAnalysis {
            game_id,
            engine_name: "Stockfish 19".to_string(),
            engine_version: "19".to_string(),
            classifier_version: crate::domain::analysis::CLASSIFIER_VERSION.to_string(),
            accuracy_method_version: crate::domain::analysis::ACCURACY_METHOD_VERSION.to_string(),
            depth: 14,
            multipv: 2,
            analyzed_at: "2026-01-01T00:00:00Z".to_string(),
            white_accuracy: Some(white_accuracy),
            black_accuracy: Some(black_accuracy),
            white_acpl: Some(20.0),
            black_acpl: Some(30.0),
            white_perf_elo: Some(2000),
            black_perf_elo: Some(1800),
            white_opening_acc: Some(white_accuracy),
            white_middlegame_acc: None,
            white_endgame_acc: None,
            black_opening_acc: Some(black_accuracy),
            black_middlegame_acc: None,
            black_endgame_acc: None,
            counts,
            summary_text: "Тестовое резюме".to_string(),
            key_moments_count: 1,
        },
        moves,
    }
}
