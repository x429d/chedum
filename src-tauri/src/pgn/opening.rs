use crate::eco::{name_from_eco_url, normalize_opening_name, EcoBook};
use crate::pgn::parser::ParsedGame;

/// заголовки PGN (ECO/Opening/ECOUrl), нормализованные.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningIdentification {
    pub eco: Option<String>,
    pub name: Option<String>,
    pub ply: Option<i32>,
    pub source: Option<String>,
}

pub fn identify_opening(parsed: &ParsedGame) -> OpeningIdentification {
    let computed = EcoBook::global().identify_with_ply(&parsed.fens_after());
    match computed {
        Some(found) => OpeningIdentification {
            eco: Some(found.eco),
            name: Some(found.name),
            ply: Some(found.ply as i32),
            source: Some("position".to_string()),
        },
        None => {
            // Запасной вариант — только заголовки PGN, причём ECO и имя
            // берутся из одного источника, чтобы не склеить разные дебюты.
            let name = parsed
                .tag("Opening")
                .and_then(normalize_opening_name)
                .or_else(|| parsed.tag("ECOUrl").and_then(name_from_eco_url));
            let eco = parsed
                .tag("ECO")
                .map(|value| value.trim().to_uppercase())
                .filter(|value| !value.is_empty());
            let source = if name.is_some() || eco.is_some() {
                Some("pgn_header".to_string())
            } else {
                None
            };
            OpeningIdentification {
                eco,
                name,
                ply: None,
                source,
            }
        }
    }
}
