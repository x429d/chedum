#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedInfo {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub multipv: u32,
    pub cp: Option<i32>,
    pub mate: Option<i32>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub pv: Vec<String>,
}

/// Разбирает строку `info ...` протокола UCI.
pub fn parse_info(line: &str) -> Option<ParsedInfo> {
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "info" {
        return None;
    }
    let rest: Vec<&str> = tokens.collect();
    let mut info = ParsedInfo {
        multipv: 1,
        ..ParsedInfo::default()
    };
    let mut index = 0;
    while index < rest.len() {
        match rest[index] {
            "depth" => {
                info.depth = rest.get(index + 1).and_then(|value| value.parse().ok());
                index += 2;
            }
            "seldepth" => {
                info.seldepth = rest.get(index + 1).and_then(|value| value.parse().ok());
                index += 2;
            }
            "multipv" => {
                info.multipv = rest
                    .get(index + 1)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1);
                index += 2;
            }
            "nodes" => {
                info.nodes = rest.get(index + 1).and_then(|value| value.parse().ok());
                index += 2;
            }
            "nps" => {
                info.nps = rest.get(index + 1).and_then(|value| value.parse().ok());
                index += 2;
            }
            "score" => {
                let kind = rest.get(index + 1).copied().unwrap_or_default();
                let value = rest
                    .get(index + 2)
                    .and_then(|value| value.parse::<i32>().ok());
                match kind {
                    "cp" => info.cp = value,
                    "mate" => info.mate = value,
                    _ => {}
                }
                index += 3;
            }
            "pv" => {
                info.pv = rest[index + 1..]
                    .iter()
                    .filter(|token| !token.starts_with("bmc"))
                    .map(|token| token.to_string())
                    .collect();
                break;
            }
            _ => index += 1,
        }
    }
    if info.pv.is_empty() {
        return None;
    }
    Some(info)
}

/// Разбирает `bestmove e2e4 ponder e7e5`.
pub fn parse_bestmove(line: &str) -> Option<String> {
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "bestmove" {
        return None;
    }
    let best = tokens.next()?;
    if best == "(none)" || best == "none" {
        None
    } else {
        Some(best.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_info_line() {
        let line = "info depth 18 seldepth 24 multipv 2 score cp -35 nodes 123456 nps 800000 hashfull 12 tbhits 0 time 154 pv e2e4 e7e5 g1f3";
        let info = parse_info(line).unwrap();
        assert_eq!(info.depth, Some(18));
        assert_eq!(info.seldepth, Some(24));
        assert_eq!(info.multipv, 2);
        assert_eq!(info.cp, Some(-35));
        assert_eq!(info.mate, None);
        assert_eq!(info.nodes, Some(123456));
        assert_eq!(info.nps, Some(800000));
        assert_eq!(info.pv, vec!["e2e4", "e7e5", "g1f3"]);
    }

    #[test]
    fn parses_mate_score() {
        let line = "info depth 12 score mate 3 multipv 1 pv h5f7 e8f7";
        let info = parse_info(line).unwrap();
        assert_eq!(info.mate, Some(3));
        assert_eq!(info.cp, None);
    }

    #[test]
    fn ignores_non_info_lines() {
        assert_eq!(parse_info("bestmove e2e4"), None);
        assert_eq!(
            parse_info("info string NNUE evaluation using nn-abc.nnue"),
            None
        );
    }

    #[test]
    fn parses_bestmove_with_ponder() {
        assert_eq!(
            parse_bestmove("bestmove e2e4 ponder e7e5"),
            Some("e2e4".to_string())
        );
        assert_eq!(parse_bestmove("bestmove (none)"), None);
        assert_eq!(parse_bestmove("info depth 1"), None);
    }
}
