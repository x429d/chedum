use shakmaty::{Chess, Position};

use chrono::Utc;

use crate::analytics::{
    average_cp_loss, game_accuracy, performance_elo, phase_accuracy, spent_time,
    summary_from_analysis, time_flag, Score, SummaryContext, TimeContext,
};
use crate::domain::analysis::{CategoryCounts, GameAnalysis, MoveCategory, Phase, TimeFlag};
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{Color, Game, TimeClass};
use crate::domain::position::{
    is_checkmate, is_hanging_after, material_value, parse_fen, parse_uci, play_uci, side_to_move,
};

pub fn phase_index(phase: Phase) -> usize {
    match phase {
        Phase::Opening => 0,
        Phase::Middlegame => 1,
        Phase::Endgame => 2,
    }
}

/// Материал игрока после первых `plies` ходов линии (UCI).
pub fn material_after_line(
    fen: &str,
    uci_moves: &[String],
    plies: usize,
    player: Color,
) -> Option<i32> {
    let mut pos = parse_fen(fen).ok()?;
    for uci in uci_moves.iter().filter(|uci| !uci.is_empty()).take(plies) {
        play_uci(&mut pos, uci).ok()?;
    }
    Some(material_value(&pos, player))
}

/// Win% стороны `player` в терминальной позиции (мат/пат).
pub fn terminal_win_prob(fen: &str, player: Color) -> Option<f64> {
    let pos = parse_fen(fen).ok()?;
    if is_checkmate(&pos) {
        let mated = side_to_move(&pos);
        Some(if mated == player { 0.0 } else { 100.0 })
    } else if pos.is_stalemate() {
        Some(50.0)
    } else {
        None
    }
}

pub fn is_terminal(fen: &str) -> bool {
    parse_fen(fen)
        .map(|pos| pos.is_checkmate() || pos.is_stalemate())
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialProfile {
    pub sacrifice: bool,
    pub material_loss_cp: i32,
    pub best_material_gain_cp: i32,
}

/// Сравнивает материальные последствия сделанного хода и главной линии.
#[allow(clippy::too_many_arguments)]
pub fn material_profile(
    before_pos: &Chess,
    fen_before: &str,
    played_uci: &str,
    fen_after: &str,
    reply: Option<&str>,
    best_uci_moves: &[String],
    player: Color,
) -> MaterialProfile {
    let before_material = material_value(before_pos, player);
    let mut played_line = vec![played_uci.to_string()];
    if let Some(reply) = reply {
        played_line.push(reply.to_string());
    }
    let played_material =
        material_after_line(fen_before, &played_line, 2, player).unwrap_or(before_material);
    let hanging = parse_uci(fen_before, played_uci)
        .ok()
        .and_then(|(_, mv)| {
            parse_fen(fen_after)
                .ok()
                .map(|pos| is_hanging_after(&pos, mv, player))
        })
        .unwrap_or(false);
    let sacrifice = played_material < before_material - 50 || hanging;
    let best_material =
        material_after_line(fen_before, best_uci_moves, 2, player).unwrap_or(before_material);
    let best_material_gain = material_after_line(fen_before, best_uci_moves, 4, player)
        .map(|value| (value - before_material) * 100)
        .unwrap_or(0)
        .max(0);
    MaterialProfile {
        sacrifice,
        material_loss_cp: (best_material - played_material).max(0) * 100,
        best_material_gain_cp: best_material_gain,
    }
}

pub struct AfterEval {
    pub win_prob_white: f64,
    pub played_win: f64,
    pub eval_cp: Option<i32>,
    pub eval_mate: Option<i32>,
    pub played_score_player: Score,
    pub player_mate: bool,
    pub allowed_mate: bool,
}

/// Оценка позиции после сделанного хода, включая терминальные позиции.
pub fn after_evaluation(
    fen_after: &str,
    after_score: Option<Score>,
    after_side: Color,
    player: Color,
    best_win: f64,
) -> AppResult<AfterEval> {
    let after_pos = parse_fen(fen_after).ok();
    let terminal_white = if after_score.is_none() {
        terminal_win_prob(fen_after, Color::White)
    } else {
        None
    };
    let terminal_player = if after_score.is_none() {
        terminal_win_prob(fen_after, player)
    } else {
        None
    };
    let (win_prob_white, played_win, white_after) = match after_score {
        Some(score) => (
            score.win_prob_white(after_side),
            100.0 - score.win_prob(),
            score.white_relative(after_side),
        ),
        None => {
            let white = terminal_white
                .ok_or_else(|| AppError::Engine("Нет оценки после хода".to_string()))?;
            (white, terminal_player.unwrap_or(50.0), Score::default())
        }
    };
    let played_score_player = after_score.map(|score| score.flipped()).unwrap_or_default();
    let player_mate = played_score_player.mate.is_some_and(|mate| mate > 0)
        || after_pos
            .as_ref()
            .is_some_and(|pos| is_checkmate(pos) && side_to_move(pos) != player);
    Ok(AfterEval {
        win_prob_white,
        played_win,
        eval_cp: if after_score.is_some() {
            white_after.cp
        } else {
            None
        },
        eval_mate: if after_score.is_some() {
            white_after.mate
        } else {
            None
        },
        played_score_player,
        player_mate,
        allowed_mate: played_score_player.mate.is_some_and(|mate| mate < 0) && best_win >= 20.0,
    })
}

pub struct TimeInfo {
    pub spent: Option<f64>,
    pub flag: TimeFlag,
}

/// Время на ход и флаг тайм-менеджмента.
pub fn time_info(
    previous_clock: Option<f64>,
    current_clock: Option<f64>,
    increment: u32,
    initial_secs: f64,
    time_class: Option<TimeClass>,
    category: MoveCategory,
) -> TimeInfo {
    let spent = spent_time(previous_clock, current_clock, f64::from(increment));
    let flag = time_flag(TimeContext {
        remaining_sec: current_clock,
        spent_sec: spent,
        initial_sec: initial_secs,
        time_class,
        category,
    });
    TimeInfo { spent, flag }
}

pub struct AggregateInput<'a> {
    pub game: &'a Game,
    pub engine_name: &'a str,
    pub engine_version: &'a str,
    pub depth: u32,
    pub multipv: u32,
    pub counts: CategoryCounts,
    pub key_moments: u32,
    pub accuracy_samples: Vec<(f64, f64)>,
    pub first_color: Color,
    pub phase_samples: [Vec<(f64, f64)>; 3],
    pub phase_first: [Option<Color>; 3],
    pub white_cp: Vec<f64>,
    pub black_cp: Vec<f64>,
    pub user_time_trouble: u32,
    pub user_impulsive: u32,
}

/// Собирает итоговые метрики партии из накопленных данных по ходам.
pub fn finalize(input: AggregateInput<'_>) -> GameAnalysis {
    let [white_accuracy, black_accuracy] = game_accuracy(&input.accuracy_samples, input.first_color);
    let white_acpl = average_cp_loss(input.white_cp);
    let black_acpl = average_cp_loss(input.black_cp);
    let [white_phase_acc, black_phase_acc] = phase_accuracy(&input.phase_samples, &input.phase_first);
    let mut analysis = GameAnalysis {
        game_id: input.game.id,
        engine_name: input.engine_name.to_string(),
        engine_version: input.engine_version.to_string(),
        classifier_version: crate::domain::analysis::CLASSIFIER_VERSION.to_string(),
        accuracy_method_version: crate::domain::analysis::ACCURACY_METHOD_VERSION.to_string(),
        depth: input.depth as i32,
        multipv: input.multipv as i32,
        analyzed_at: Utc::now().to_rfc3339(),
        white_accuracy,
        black_accuracy,
        white_acpl,
        black_acpl,
        white_perf_elo: performance_elo(input.game.white_elo, white_accuracy, white_acpl),
        black_perf_elo: performance_elo(input.game.black_elo, black_accuracy, black_acpl),
        white_opening_acc: white_phase_acc[0],
        white_middlegame_acc: white_phase_acc[1],
        white_endgame_acc: white_phase_acc[2],
        black_opening_acc: black_phase_acc[0],
        black_middlegame_acc: black_phase_acc[1],
        black_endgame_acc: black_phase_acc[2],
        counts: input.counts,
        summary_text: String::new(),
        key_moments_count: input.key_moments as i32,
    };
    let context = SummaryContext::from_game(input.game);
    analysis.summary_text = summary_from_analysis(
        &context,
        &analysis,
        input.user_time_trouble,
        input.user_impulsive,
    );
    analysis
}
