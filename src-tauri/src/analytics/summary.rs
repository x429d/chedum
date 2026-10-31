use crate::domain::analysis::{GameAnalysis, MoveCategory, Phase};
use crate::domain::game::{Color, UserColor};

fn category_en(category: MoveCategory) -> &'static str {
    match category {
        MoveCategory::Book => "theory",
        MoveCategory::Forced => "forced moves",
        MoveCategory::Brilliant => "brilliant moves",
        MoveCategory::Great => "great moves",
        MoveCategory::Best => "best moves",
        MoveCategory::Excellent => "excellent moves",
        MoveCategory::Good => "good moves",
        MoveCategory::Miss => "missed chances",
        MoveCategory::Inaccuracy => "inaccuracies",
        MoveCategory::Mistake => "mistakes",
        MoveCategory::Blunder => "blunders",
    }
}

fn phase_en(phase: Phase) -> &'static str {
    match phase {
        Phase::Opening => "opening",
        Phase::Middlegame => "middlegame",
        Phase::Endgame => "endgame",
    }
}

fn fmt1(value: f64) -> String {
    format!("{value:.1}")
}

/// Fields needed for the summary. Keeping this separate lets the text be built
/// both from the domain model and from a SQL row without re-reading the game.
pub struct SummaryContext<'a> {
    pub white: &'a str,
    pub black: &'a str,
    pub result: &'a str,
    pub user_color: UserColor,
    pub eco: Option<&'a str>,
    pub opening: Option<&'a str>,
}

impl<'a> SummaryContext<'a> {
    pub fn from_game(game: &'a crate::domain::game::Game) -> Self {
        SummaryContext {
            white: &game.white_username,
            black: &game.black_username,
            result: &game.result,
            user_color: game.user_color,
            eco: game.eco.as_deref(),
            opening: game.opening_name.as_deref(),
        }
    }
}

/// The game summary is derived only from `GameAnalysis` fields, never
/// recalculated from moves, so it cannot disagree with the accuracy cards.
pub fn summary_from_analysis(
    context: &SummaryContext<'_>,
    analysis: &GameAnalysis,
    time_trouble: u32,
    impulsive_errors: u32,
) -> String {
    let mut parts = Vec::new();
    let opening = match (context.eco, context.opening) {
        (Some(eco), Some(name)) => Some(format!("{eco} «{name}»")),
        (Some(eco), None) => Some(eco.to_string()),
        (None, Some(name)) => Some(name.to_string()),
        (None, None) => None,
    };
    if let Some(opening) = opening {
        parts.push(format!("Opening: {opening}."));
    }
    let user_color = context.user_color.color();
    match (user_color, context.result) {
        (Some(Color::White), "1-0") => {
            parts.push(format!("You played White and beat {}.", context.black))
        }
        (Some(Color::White), "0-1") => {
            parts.push(format!("You played White and lost to {}.", context.black))
        }
        (Some(Color::Black), "0-1") => {
            parts.push(format!("You played Black and beat {}.", context.white))
        }
        (Some(Color::Black), "1-0") => {
            parts.push(format!("You played Black and lost to {}.", context.white))
        }
        (_, "1/2-1/2") => parts.push(format!(
            "{} — {} ended in a draw.",
            context.white, context.black
        )),
        _ => parts.push(format!(
            "{} — {} ended {}.",
            context.white, context.black, context.result
        )),
    }
    let accuracy = match user_color {
        Some(Color::White) => analysis.white_accuracy,
        Some(Color::Black) => analysis.black_accuracy,
        None => None,
    };
    let acpl = match user_color {
        Some(Color::White) => analysis.white_acpl,
        Some(Color::Black) => analysis.black_acpl,
        None => None,
    };
    if let Some(accuracy) = accuracy {
        let acpl_text = acpl
            .map(|value| format!(", ACPL {}", fmt1(value)))
            .unwrap_or_default();
        parts.push(format!("Your move accuracy was {accuracy:.1}%{acpl_text}."));
    }
    if let Some(color) = user_color {
        let errors: Vec<String> = [
            MoveCategory::Blunder,
            MoveCategory::Mistake,
            MoveCategory::Miss,
            MoveCategory::Inaccuracy,
        ]
        .iter()
        .filter_map(|category| {
            let count = analysis.counts.get(color, *category);
            if count > 0 {
                Some(format!("{}: {count}", category_en(*category)))
            } else {
                None
            }
        })
        .collect();
        if errors.is_empty() {
            parts.push("No serious errors were found — a clean game.".to_string());
        } else {
            parts.push(format!("Weak spot: {}.", errors.join(", ")));
        }
        let phases = [Phase::Opening, Phase::Middlegame, Phase::Endgame];
        let user_phases = user_phase_accuracies(analysis, color);
        let weakest = phases
            .iter()
            .zip(user_phases.iter())
            .filter_map(|(phase, accuracy)| accuracy.map(|value| (*phase, value)))
            .min_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((phase, value)) = weakest {
            parts.push(format!(
                "Lowest accuracy in the {}: {}%.",
                phase_en(phase),
                fmt1(value)
            ));
        }
    }
    if analysis.key_moments_count > 0 {
        parts.push(format!(
            "Key moments: {}. Time-trouble moves: {}, impulsive errors: {}.",
            analysis.key_moments_count, time_trouble, impulsive_errors
        ));
    } else {
        parts.push("No key turning points were recorded.".to_string());
    }
    parts.join(" ")
}

fn user_phase_accuracies(analysis: &GameAnalysis, color: Color) -> [Option<f64>; 3] {
    match color {
        Color::White => [
            analysis.white_opening_acc,
            analysis.white_middlegame_acc,
            analysis.white_endgame_acc,
        ],
        Color::Black => [
            analysis.black_opening_acc,
            analysis.black_middlegame_acc,
            analysis.black_endgame_acc,
        ],
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::analysis::CategoryCounts;

    fn analysis() -> GameAnalysis {
        let mut counts = CategoryCounts::new();
        counts.increment(Color::White, MoveCategory::Blunder);
        counts.increment(Color::White, MoveCategory::Mistake);
        counts.increment(Color::White, MoveCategory::Mistake);
        GameAnalysis {
            game_id: 1,
            engine_name: "Stockfish 19".to_string(),
            engine_version: "19".to_string(),
            classifier_version: crate::domain::analysis::CLASSIFIER_VERSION.to_string(),
            accuracy_method_version: crate::domain::analysis::ACCURACY_METHOD_VERSION.to_string(),
            depth: 14,
            multipv: 2,
            analyzed_at: String::new(),
            white_accuracy: Some(63.42),
            black_accuracy: Some(59.24),
            white_acpl: Some(101.6),
            black_acpl: Some(87.4),
            white_perf_elo: None,
            black_perf_elo: None,
            white_opening_acc: Some(80.0),
            white_middlegame_acc: Some(68.4),
            white_endgame_acc: Some(72.5),
            black_opening_acc: Some(70.0),
            black_middlegame_acc: Some(66.0),
            black_endgame_acc: Some(70.0),
            counts,
            summary_text: String::new(),
            key_moments_count: 5,
        }
    }

    fn context() -> SummaryContext<'static> {
        SummaryContext {
            white: "Wakanov",
            black: "KKolobe",
            result: "1-0",
            user_color: UserColor::White,
            eco: Some("C25"),
            opening: Some("Vienna Game"),
        }
    }

    #[test]
    fn summary_numbers_match_analysis_fields() {
        let analysis = analysis();
        let text = summary_from_analysis(&context(), &analysis, 2, 1);
        assert!(text.contains("63.4%"), "{text}");
        assert!(text.contains("ACPL 101.6"), "{text}");
        assert!(text.contains("blunders: 1"), "{text}");
        assert!(text.contains("mistakes: 2"), "{text}");
        assert!(text.contains("middlegame: 68.4%"), "{text}");
        assert!(text.contains("Key moments: 5"), "{text}");
        assert!(
            text.contains("Time-trouble moves: 2, impulsive errors: 1"),
            "{text}"
        );
        assert!(!text.contains("74.6"), "{text}");
    }

    #[test]
    fn regenerated_summary_follows_updated_accuracy() {
        let mut analysis = analysis();
        analysis.white_accuracy = Some(51.26);
        let text = summary_from_analysis(&context(), &analysis, 0, 0);
        assert!(text.contains("51.3%"), "{text}");
        assert!(!text.contains("63.4"), "{text}");
    }

    #[test]
    fn summary_without_user_color_has_no_accuracy_line() {
        let mut analysis = analysis();
        analysis.white_accuracy = Some(90.0);
        let context = SummaryContext {
            user_color: UserColor::None,
            ..context()
        };
        let text = summary_from_analysis(&context, &analysis, 0, 0);
        assert!(!text.contains("Your move accuracy"), "{text}");
    }
}
