use crate::domain::analysis::MoveCategory;

pub struct CoachInput<'a> {
    pub category: MoveCategory,
    pub played_san: &'a str,
    pub best_san: Option<&'a str>,
    pub allowed_mate_in: Option<u32>,
    pub missed_mate_in: Option<u32>,
    pub eval_loss_pawns: f64,
    pub material_loss_cp: i32,
    pub best_material_gain_cp: i32,
    pub king_safety_loss: bool,
    pub sacrifice: bool,
    pub pv_san: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub tag: String,
    pub text: String,
}

fn best(input: &CoachInput<'_>) -> String {
    input
        .best_san
        .map(str::to_string)
        .unwrap_or_else(|| "сильнейшее продолжение".to_string())
}

fn pv(input: &CoachInput<'_>) -> String {
    let line = input
        .pv_san
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if line.is_empty() {
        "—".to_string()
    } else {
        line
    }
}

/// Генерирует понятное объяснение хода на русском языке.
pub fn explain(input: &CoachInput<'_>) -> Option<Explanation> {
    if !input.category.needs_coach() {
        return None;
    }
    if matches!(
        input.category,
        MoveCategory::Brilliant | MoveCategory::Great
    ) {
        let text = if input.sacrifice {
            format!(
                "Бриллиантовая жертва: {} отдаёт материал, но сохраняет сильную оценку. Главная линия: {}.",
                input.played_san,
                pv(input)
            )
        } else {
            format!(
                "Сильнейшее продолжение {}. {} сохраняет перевес; ключевая линия: {}.",
                input.played_san,
                best(input),
                pv(input)
            )
        };
        let tag = if input.sacrifice {
            "brilliant_sacrifice"
        } else {
            "strong_continuation"
        };
        return Some(Explanation {
            tag: tag.to_string(),
            text,
        });
    }
    if let Some(distance) = input.allowed_mate_in {
        return Some(Explanation {
            tag: "allowed_mate".to_string(),
            text: format!(
                "Позволяет сопернику объявить форсированный мат в {distance} ходов. Вместо {} точнее было {}.",
                input.played_san,
                best(input)
            ),
        });
    }
    if let Some(distance) = input.missed_mate_in {
        return Some(Explanation {
            tag: "missed_mate".to_string(),
            text: format!(
                "Упущен форсированный мат в {distance} ходов. Вместо {} выигрывало {}.",
                input.played_san,
                best(input)
            ),
        });
    }
    if input.material_loss_cp >= 300 {
        let pawns = f64::from(input.material_loss_cp) / 100.0;
        return Some(Explanation {
            tag: "hung_piece".to_string(),
            text: format!(
                "Потеря материала: после {} фигура остаётся без защиты, главная линия соперника даёт не менее {pawns:.1} пешки перевеса. Точнее было {}.",
                input.played_san,
                best(input)
            ),
        });
    }
    if input.best_material_gain_cp >= 200 {
        let pawns = f64::from(input.best_material_gain_cp) / 100.0;
        return Some(Explanation {
            tag: "missed_tactic".to_string(),
            text: format!(
                "Упущен тактический удар: линия {} давала выигрыш материала около {pawns:.1} пешки.",
                best(input)
            ),
        });
    }
    if input.king_safety_loss {
        return Some(Explanation {
            tag: "king_safety".to_string(),
            text: format!(
                "Ослабление прикрытия короля: {} создаёт прямые угрозы, соперник отвечает шахом. Надёжнее было {}.",
                input.played_san,
                best(input)
            ),
        });
    }
    Some(Explanation {
        tag: "positional_concession".to_string(),
        text: format!(
            "Позиционная уступка: потеря {:.1} оценки и контроля над ключевыми полями. Точнее было {}.",
            input.eval_loss_pawns.max(0.0),
            best(input)
        ),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn input<'a>(category: MoveCategory, pv_san: &'a [String]) -> CoachInput<'a> {
        CoachInput {
            category,
            played_san: "Qh5",
            best_san: Some("Nf3"),
            allowed_mate_in: None,
            missed_mate_in: None,
            eval_loss_pawns: 1.2,
            material_loss_cp: 0,
            best_material_gain_cp: 0,
            king_safety_loss: false,
            sacrifice: false,
            pv_san,
        }
    }

    #[test]
    fn allowed_mate_has_priority() {
        let line = vec!["Qh5".to_string()];
        let mut ctx = input(MoveCategory::Blunder, &line);
        ctx.allowed_mate_in = Some(3);
        let explanation = explain(&ctx).unwrap();
        assert_eq!(explanation.tag, "allowed_mate");
        assert!(explanation.text.contains("мат в 3 ходов"));
    }

    #[test]
    fn missed_mate_is_reported() {
        let line = vec!["Qh5".to_string()];
        let mut ctx = input(MoveCategory::Miss, &line);
        ctx.missed_mate_in = Some(2);
        assert_eq!(explain(&ctx).unwrap().tag, "missed_mate");
    }

    #[test]
    fn hung_piece_and_tactic_tags() {
        let line = vec!["Qh5".to_string()];
        let mut ctx = input(MoveCategory::Mistake, &line);
        ctx.material_loss_cp = 400;
        assert_eq!(explain(&ctx).unwrap().tag, "hung_piece");
        ctx.material_loss_cp = 0;
        ctx.best_material_gain_cp = 300;
        assert_eq!(explain(&ctx).unwrap().tag, "missed_tactic");
    }

    #[test]
    fn king_safety_and_positional_tags() {
        let line = vec!["Qh5".to_string()];
        let mut ctx = input(MoveCategory::Inaccuracy, &line);
        ctx.king_safety_loss = true;
        assert_eq!(explain(&ctx).unwrap().tag, "king_safety");
        ctx.king_safety_loss = false;
        assert_eq!(explain(&ctx).unwrap().tag, "positional_concession");
    }

    #[test]
    fn positive_categories_get_positive_explanations() {
        let line = vec!["Nxb5".to_string()];
        let mut ctx = input(MoveCategory::Brilliant, &line);
        ctx.sacrifice = true;
        assert_eq!(explain(&ctx).unwrap().tag, "brilliant_sacrifice");
        ctx.sacrifice = false;
        ctx.category = MoveCategory::Great;
        assert_eq!(explain(&ctx).unwrap().tag, "strong_continuation");
    }
}
