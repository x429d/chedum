use std::collections::BTreeMap;

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

/// Structured explanation. The UI renders it from the tag and parameters in
/// the selected language; no localized prose is stored in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub tag: String,
    pub params: BTreeMap<String, String>,
}

fn param(key: &str, value: impl Into<String>) -> (String, String) {
    (key.to_string(), value.into())
}

fn line(input: &CoachInput<'_>) -> String {
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

fn base_params(input: &CoachInput<'_>) -> BTreeMap<String, String> {
    BTreeMap::from([
        param("playedSan", input.played_san),
        param("bestSan", input.best_san.unwrap_or("—")),
        param("line", line(input)),
    ])
}

/// Builds a structured explanation for moves that need coaching.
pub fn explain(input: &CoachInput<'_>) -> Option<Explanation> {
    if !input.category.needs_coach() {
        return None;
    }
    if matches!(
        input.category,
        MoveCategory::Brilliant | MoveCategory::Great
    ) {
        let tag = if input.sacrifice {
            "brilliant_sacrifice"
        } else {
            "strong_continuation"
        };
        return Some(Explanation {
            tag: tag.to_string(),
            params: base_params(input),
        });
    }
    if let Some(distance) = input.allowed_mate_in {
        let mut params = base_params(input);
        params.insert("distance".to_string(), distance.to_string());
        return Some(Explanation {
            tag: "allowed_mate".to_string(),
            params,
        });
    }
    if let Some(distance) = input.missed_mate_in {
        let mut params = base_params(input);
        params.insert("distance".to_string(), distance.to_string());
        return Some(Explanation {
            tag: "missed_mate".to_string(),
            params,
        });
    }
    if input.material_loss_cp >= 300 {
        let mut params = base_params(input);
        params.insert(
            "pawns".to_string(),
            format!("{:.1}", f64::from(input.material_loss_cp) / 100.0),
        );
        return Some(Explanation {
            tag: "hung_piece".to_string(),
            params,
        });
    }
    if input.best_material_gain_cp >= 200 {
        let mut params = base_params(input);
        params.insert(
            "pawns".to_string(),
            format!("{:.1}", f64::from(input.best_material_gain_cp) / 100.0),
        );
        return Some(Explanation {
            tag: "missed_tactic".to_string(),
            params,
        });
    }
    if input.king_safety_loss {
        return Some(Explanation {
            tag: "king_safety".to_string(),
            params: base_params(input),
        });
    }
    let mut params = base_params(input);
    params.insert(
        "pawns".to_string(),
        format!("{:.1}", input.eval_loss_pawns.max(0.0)),
    );
    Some(Explanation {
        tag: "positional_concession".to_string(),
        params,
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
        assert_eq!(explanation.params.get("distance"), Some(&"3".to_string()));
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
        let explanation = explain(&ctx).unwrap();
        assert_eq!(explanation.tag, "hung_piece");
        assert_eq!(explanation.params.get("pawns"), Some(&"4.0".to_string()));
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
