use crate::domain::analysis::{CategoryCounts, MoveCategory, Phase};
use crate::domain::game::{Color, UserColor};

fn category_ru(category: MoveCategory) -> &'static str {
    match category {
        MoveCategory::Book => "теория",
        MoveCategory::Forced => "единственный ход",
        MoveCategory::Brilliant => "бриллиантовые ходы",
        MoveCategory::Great => "отличные ходы",
        MoveCategory::Best => "лучшие ходы",
        MoveCategory::Excellent => "прекрасные ходы",
        MoveCategory::Good => "хорошие ходы",
        MoveCategory::Miss => "упущенные шансы",
        MoveCategory::Inaccuracy => "неточности",
        MoveCategory::Mistake => "ошибки",
        MoveCategory::Blunder => "зевки",
    }
}

fn phase_ru(phase: Phase) -> &'static str {
    match phase {
        Phase::Opening => "дебюте",
        Phase::Middlegame => "миттельшпиле",
        Phase::Endgame => "эндшпиле",
    }
}

fn fmt1(value: f64) -> String {
    format!("{value:.1}")
}

pub struct SummaryInput<'a> {
    pub white: &'a str,
    pub black: &'a str,
    pub result: &'a str,
    pub user_color: UserColor,
    pub white_accuracy: Option<f64>,
    pub black_accuracy: Option<f64>,
    pub white_acpl: Option<f64>,
    pub black_acpl: Option<f64>,
    pub counts: &'a CategoryCounts,
    pub eco: Option<&'a str>,
    pub opening: Option<&'a str>,
    pub key_moments: u32,
    pub user_phases: [Option<f64>; 3],
    pub time_trouble: u32,
    pub impulsive_errors: u32,
}

/// Детерминированное текстовое резюме партии для вкладки «Отчёт».
pub fn build_summary(input: &SummaryInput<'_>) -> String {
    let mut parts = Vec::new();
    let opening = match (input.eco, input.opening) {
        (Some(eco), Some(name)) => Some(format!("{eco} «{name}»")),
        (Some(eco), None) => Some(eco.to_string()),
        (None, Some(name)) => Some(name.to_string()),
        (None, None) => None,
    };
    if let Some(opening) = opening {
        parts.push(format!("Дебют: {opening}."));
    }
    let user_color = input.user_color.color();
    match (user_color, input.result) {
        (Some(Color::White), "1-0") => parts.push(format!(
            "Вы играли белыми и победили соперника {}.",
            input.black
        )),
        (Some(Color::White), "0-1") => parts.push(format!(
            "Вы играли белыми и уступили сопернику {}.",
            input.black
        )),
        (Some(Color::Black), "0-1") => parts.push(format!(
            "Вы играли чёрными и победили соперника {}.",
            input.white
        )),
        (Some(Color::Black), "1-0") => parts.push(format!(
            "Вы играли чёрными и уступили сопернику {}.",
            input.white
        )),
        (_, "1/2-1/2") => parts.push(format!(
            "Партия {} — {} завершилась вничью.",
            input.white, input.black
        )),
        _ => parts.push(format!(
            "Партия {} — {} завершилась со счётом {}.",
            input.white, input.black, input.result
        )),
    }
    let accuracy = match user_color {
        Some(Color::White) => input.white_accuracy,
        Some(Color::Black) => input.black_accuracy,
        None => None,
    };
    let acpl = match user_color {
        Some(Color::White) => input.white_acpl,
        Some(Color::Black) => input.black_acpl,
        None => None,
    };
    if let Some(accuracy) = accuracy {
        let acpl_text = acpl
            .map(|value| format!(", ACPL {}", fmt1(value)))
            .unwrap_or_default();
        parts.push(format!("Точность ваших ходов — {accuracy:.1}%{acpl_text}."));
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
            let count = input.counts.get(color, *category);
            if count > 0 {
                Some(format!("{}: {count}", category_ru(*category)))
            } else {
                None
            }
        })
        .collect();
        if errors.is_empty() {
            parts.push("Грубых ошибок не обнаружено — партия проведена чисто.".to_string());
        } else {
            parts.push(format!("Слабое место — {}.", errors.join(", ")));
        }
        let phases = [Phase::Opening, Phase::Middlegame, Phase::Endgame];
        let weakest = phases
            .iter()
            .zip(input.user_phases.iter())
            .filter_map(|(phase, accuracy)| accuracy.map(|value| (*phase, value)))
            .min_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((phase, value)) = weakest {
            parts.push(format!(
                "Наименьшая точность в {} — {}%.",
                phase_ru(phase),
                fmt1(value)
            ));
        }
    }
    if input.key_moments > 0 {
        parts.push(format!(
            "Ключевых моментов: {}. В цейтноте пройдено ходов: {}, поспешных ошибок: {}.",
            input.key_moments, input.time_trouble, input.impulsive_errors
        ));
    } else {
        parts.push("Ключевых переломных моментов не зафиксировано.".to_string());
    }
    parts.join(" ")
}
