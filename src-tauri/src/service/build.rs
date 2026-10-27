use crate::analytics::{
    classify, explain, is_critical, move_accuracy, phase_for, ClassifyInput, CoachInput, Score,
};
use crate::domain::analysis::{AnalysisDraft, CategoryCounts, MoveCategory, MoveEvaluation};
use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{Color, Game};
use crate::domain::position::{legal_move_count, material_diff_white, parse_fen, side_to_move};
use crate::eco::EcoBook;
use crate::pgn::clocks::{parse_time_control, time_class_from_control};
use crate::pgn::ParsedGame;
use crate::service::build_helpers::{
    after_evaluation, finalize, material_profile, phase_index, time_info, AggregateInput,
};
use crate::service::evaluation::PositionEval;

pub struct BuildInput<'a> {
    pub game: &'a Game,
    pub parsed: &'a ParsedGame,
    pub evaluations: &'a [PositionEval],
    pub engine_name: &'a str,
    pub depth: u32,
    pub multipv: u32,
}

pub fn build_analysis(input: &BuildInput<'_>) -> AppResult<AnalysisDraft> {
    let total = input.parsed.moves.len();
    if total == 0 {
        return Err(AppError::Validation("В партии нет ходов".to_string()));
    }
    if input.evaluations.len() < total + 1 {
        return Err(AppError::Validation(
            "Недостаточно оценок движка для партии".to_string(),
        ));
    }
    let (base_secs, increment) = input
        .game
        .time_control
        .as_deref()
        .and_then(parse_time_control)
        .unwrap_or((0, 0));
    let initial_secs = if base_secs > 0 {
        f64::from(base_secs)
    } else {
        input
            .parsed
            .moves
            .iter()
            .filter_map(|mv| mv.clock_after)
            .fold(0.0, f64::max)
    };
    let time_class = input
        .game
        .time_class
        .or_else(|| time_class_from_control(input.game.time_control.as_deref()));
    let initial_clock = if initial_secs > 0.0 {
        Some(initial_secs)
    } else {
        None
    };
    let mut prev_clocks: [Option<f64>; 2] = [initial_clock, initial_clock];
    let mut prev_category: Option<MoveCategory> = None;
    let mut counts = CategoryCounts::new();
    let mut moves: Vec<MoveEvaluation> = Vec::with_capacity(total);
    let mut accuracy_samples: Vec<(f64, f64)> = Vec::with_capacity(total);
    let mut phase_samples: [Vec<(f64, f64)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut phase_first: [Option<Color>; 3] = [None, None, None];
    let mut first_color: Option<Color> = None;
    let mut white_cp: Vec<f64> = Vec::new();
    let mut black_cp: Vec<f64> = Vec::new();
    let mut key_moments = 0u32;
    let mut user_time_trouble = 0u32;
    let mut user_impulsive = 0u32;
    let user_color = input.game.user_color.color();

    for index in 0..total {
        let mv = &input.parsed.moves[index];
        let before = &input.evaluations[index];
        let after = &input.evaluations[index + 1];
        let before_pos = parse_fen(&mv.fen_before)?;
        let player = side_to_move(&before_pos);
        let best_line = before.lines.first().ok_or_else(|| {
            AppError::Engine(format!(
                "Движок не вернул оценку для позиции {}",
                mv.fen_before
            ))
        })?;
        let best_score = Score::new(best_line.score_cp, best_line.score_mate);
        let best_win = best_score.win_prob();
        let after_side = player.opposite();
        let after_score = after
            .lines
            .first()
            .map(|line| Score::new(line.score_cp, line.score_mate));
        let after_pos = parse_fen(&mv.fen_after).ok();
        let after_eval =
            after_evaluation(&mv.fen_after, after_score, after_side, player, best_win)?;
        let win_prob_white = after_eval.win_prob_white;
        let played_win = after_eval.played_win;
        let eval_cp = after_eval.eval_cp;
        let eval_mate = after_eval.eval_mate;
        let played_score_player = after_eval.played_score_player;
        let player_mate = after_eval.player_mate;
        let allowed_mate = after_eval.allowed_mate;
        let delta = (best_win - played_win).max(0.0);
        let second_gap = before
            .lines
            .get(1)
            .map(|line| (best_win - line.win_prob).max(0.0))
            .unwrap_or(0.0);
        let critical = is_critical(second_gap);
        let played_is_best = best_line.uci_moves.first() == Some(&mv.uci) || delta <= 0.5;
        let missed_mate = best_score.mate.is_some_and(|mate| mate > 0) && !player_mate;
        let in_book = index < 24 && EcoBook::global().lookup(&mv.fen_after).is_some();
        let reply = after
            .lines
            .first()
            .and_then(|line| line.uci_moves.first())
            .map(String::as_str);
        let material = material_profile(
            &before_pos,
            &mv.fen_before,
            &mv.uci,
            &mv.fen_after,
            reply,
            &best_line.uci_moves,
            player,
        );
        let sacrifice = material.sacrifice;
        let material_loss_cp = material.material_loss_cp;
        let best_material_gain = material.best_material_gain_cp;
        let opponent_check = after
            .lines
            .first()
            .and_then(|line| line.san_moves.first())
            .is_some_and(|san| san.ends_with('+') || san.ends_with('#'));
        let eval_loss_pawns =
            ((best_score.cp_value() - played_score_player.cp_value()).abs() / 100.0).min(99.0);
        let cp_loss = (best_score.cp_value() - played_score_player.cp_value()).max(0.0);
        let legal_moves = legal_move_count(&before_pos);
        let classification = classify(&ClassifyInput {
            ply: index as u32 + 1,
            in_book,
            legal_moves,
            delta_w: delta,
            played_is_best,
            best_win,
            played_win,
            second_gap,
            sacrifice,
            prev_opponent_blunder: prev_category.is_some_and(MoveCategory::is_mistake_or_blunder),
            missed_mate,
            allowed_mate,
        });
        let accuracy = if matches!(classification, MoveCategory::Book | MoveCategory::Forced) {
            100.0
        } else {
            move_accuracy(delta)
        };
        let side_index = match player {
            Color::White => 0,
            Color::Black => 1,
        };
        let time = time_info(
            prev_clocks[side_index],
            mv.clock_after,
            increment,
            initial_secs,
            time_class,
            classification,
        );
        let spent = time.spent;
        let flag = time.flag;
        let explanation = explain(&CoachInput {
            category: classification,
            played_san: &mv.san,
            best_san: best_line.san_moves.first().map(String::as_str),
            allowed_mate_in: played_score_player
                .mate
                .filter(|mate| *mate < 0)
                .map(|mate| mate.unsigned_abs()),
            missed_mate_in: best_score
                .mate
                .filter(|mate| *mate > 0)
                .map(|mate| mate as u32),
            eval_loss_pawns,
            material_loss_cp,
            best_material_gain_cp: best_material_gain,
            king_safety_loss: opponent_check && delta > 3.0,
            sacrifice,
            pv_san: &best_line.san_moves,
        });
        let phase = phase_for(index as u32 + 1, &mv.fen_before);
        let phase_slot = phase_index(phase);
        let sample = (best_win, played_win);
        accuracy_samples.push(sample);
        if first_color.is_none() {
            first_color = Some(player);
        }
        if phase_first[phase_slot].is_none() {
            phase_first[phase_slot] = Some(player);
        }
        phase_samples[phase_slot].push(sample);
        match player {
            Color::White => white_cp.push(cp_loss),
            Color::Black => black_cp.push(cp_loss),
        }
        counts.increment(player, classification);
        if classification.is_key_moment() || critical {
            key_moments += 1;
        }
        if user_color == Some(player) {
            if flag == crate::domain::analysis::TimeFlag::TimeTrouble {
                user_time_trouble += 1;
            }
            if flag == crate::domain::analysis::TimeFlag::ImpulsiveError {
                user_impulsive += 1;
            }
        }
        moves.push(MoveEvaluation {
            game_id: input.game.id,
            ply: index as i32 + 1,
            phase,
            fen_before: mv.fen_before.clone(),
            fen_after: mv.fen_after.clone(),
            played_uci: mv.uci.clone(),
            played_san: mv.san.clone(),
            best_uci: best_line
                .uci_moves
                .first()
                .cloned()
                .or_else(|| before.best_move.clone()),
            best_san: best_line.san_moves.first().cloned(),
            eval_cp,
            eval_mate,
            best_eval_cp: best_score.white_relative(player).cp,
            best_eval_mate: best_score.white_relative(player).mate,
            win_prob_white,
            win_prob_loss: delta,
            move_accuracy: accuracy,
            classification,
            is_critical: critical,
            material_diff_white: after_pos.as_ref().map(material_diff_white).unwrap_or(0),
            clock_remaining_sec: mv.clock_after,
            time_spent_sec: spent,
            time_flag: flag,
            explanation_tag: explanation.as_ref().map(|value| value.tag.clone()),
            explanation_text: explanation.map(|value| value.text),
            pv_lines: before.lines.clone(),
            user_comment: String::new(),
        });
        prev_clocks[side_index] = mv.clock_after.or(prev_clocks[side_index]);
        prev_category = Some(classification);
    }

    let analysis = finalize(AggregateInput {
        game: input.game,
        engine_name: input.engine_name,
        depth: input.depth,
        multipv: input.multipv,
        counts,
        key_moments,
        accuracy_samples,
        first_color: first_color.unwrap_or(Color::White),
        phase_samples,
        phase_first,
        white_cp,
        black_cp,
        user_time_trouble,
        user_impulsive,
    });
    Ok(AnalysisDraft { analysis, moves })
}
