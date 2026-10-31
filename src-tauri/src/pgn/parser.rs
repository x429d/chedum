use std::collections::BTreeMap;
use std::io::Cursor;
use std::ops::ControlFlow;

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use pgn_reader::{RawComment, RawTag, Reader, SanPlus, Skip, Visitor};
use shakmaty::Chess;

use crate::domain::error::{AppError, AppResult};
use crate::domain::game::{GameDraft, GameSource, TimeClass, UserColor};
use crate::domain::position::{move_uci, parse_fen, position_fen, san_and_play};
use crate::pgn::clocks::{parse_clock_comment, time_class_from_control};
use crate::pgn::opening::identify_opening;

#[derive(Debug, Clone)]
pub struct ParsedMove {
    pub san: String,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
    pub clock_after: Option<f64>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedGame {
    pub tags: BTreeMap<String, String>,
    pub moves: Vec<ParsedMove>,
    pub initial_fen: Option<String>,
    pub result: String,
}

impl ParsedGame {
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags.get(key).map(String::as_str)
    }

    pub fn fens_after(&self) -> Vec<String> {
        self.moves.iter().map(|mv| mv.fen_after.clone()).collect()
    }

    pub fn to_pgn(&self) -> String {
        crate::pgn::writer::game_to_pgn(self)
    }
}

struct MovetextState {
    tags: BTreeMap<String, String>,
    pos: Chess,
    moves: Vec<ParsedMove>,
    initial_fen: Option<String>,
    result: String,
}

struct GameVisitor;

impl Visitor for GameVisitor {
    type Tags = BTreeMap<String, String>;
    type Movetext = MovetextState;
    type Output = AppResult<ParsedGame>;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(BTreeMap::new())
    }

    fn tag(
        &mut self,
        tags: &mut Self::Tags,
        name: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        let key = String::from_utf8_lossy(name).to_string();
        tags.insert(key, value.decode_utf8_lossy().into_owned());
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        let initial_fen = tags
            .get("FEN")
            .filter(|value| !value.trim().is_empty())
            .cloned();
        let pos = match initial_fen.as_deref() {
            Some(fen) => match parse_fen(fen) {
                Ok(pos) => pos,
                Err(err) => return ControlFlow::Break(Err(err)),
            },
            None => Chess::default(),
        };
        let result = tags
            .get("Result")
            .cloned()
            .unwrap_or_else(|| "*".to_string());
        ControlFlow::Continue(MovetextState {
            tags,
            pos,
            moves: Vec::new(),
            initial_fen,
            result,
        })
    }

    fn san(&mut self, state: &mut Self::Movetext, san_plus: SanPlus) -> ControlFlow<Self::Output> {
        let fen_before = position_fen(&state.pos);
        let mv = match san_plus.san.to_move(&state.pos) {
            Ok(mv) => mv,
            Err(err) => {
                return ControlFlow::Break(Err(AppError::Parse(format!(
                    "Illegal move «{}»: {err}",
                    san_plus.san
                ))))
            }
        };
        let uci = move_uci(mv);
        let san = san_and_play(&mut state.pos, mv);
        let fen_after = position_fen(&state.pos);
        state.moves.push(ParsedMove {
            san,
            uci,
            fen_before,
            fen_after,
            clock_after: None,
            comments: Vec::new(),
        });
        ControlFlow::Continue(())
    }

    fn comment(
        &mut self,
        state: &mut Self::Movetext,
        comment: RawComment<'_>,
    ) -> ControlFlow<Self::Output> {
        let text = String::from_utf8_lossy(comment.as_bytes()).into_owned();
        if let Some(last) = state.moves.last_mut() {
            if last.clock_after.is_none() {
                last.clock_after = parse_clock_comment(&text);
            }
            if !text.trim().is_empty() {
                last.comments.push(text);
            }
        }
        ControlFlow::Continue(())
    }

    fn begin_variation(&mut self, _state: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {
        ControlFlow::Continue(Skip(true))
    }

    fn end_game(&mut self, state: Self::Movetext) -> Self::Output {
        Ok(ParsedGame {
            tags: state.tags,
            moves: state.moves,
            initial_fen: state.initial_fen,
            result: state.result,
        })
    }
}

#[derive(Debug, Default)]
pub struct ParseReport {
    pub games: Vec<ParsedGame>,
    pub errors: Vec<String>,
}

/// Streams every game in a PGN, continuing past individual errors.
pub fn parse_pgn(text: &str) -> ParseReport {
    let mut reader = Reader::new(Cursor::new(text.as_bytes()));
    let mut visitor = GameVisitor;
    let mut report = ParseReport::default();
    loop {
        match reader.read_game(&mut visitor) {
            Ok(Some(Ok(game))) => report.games.push(game),
            Ok(Some(Err(err))) => report.errors.push(err.to_string()),
            Ok(None) => break,
            Err(err) => {
                report.errors.push(format!("Could not read PGN: {err}"));
                break;
            }
        }
    }
    report
}

pub fn parse_single(text: &str) -> AppResult<ParsedGame> {
    let report = parse_pgn(text);
    if let Some(game) = report.games.into_iter().next() {
        return Ok(game);
    }
    Err(AppError::Parse(
        report
            .errors
            .into_iter()
            .next()
            .unwrap_or_else(|| "No games found in the PGN".to_string()),
    ))
}

fn parse_elo(tags: &BTreeMap<String, String>, key: &str) -> Option<i32> {
    tags.get(key)
        .and_then(|value| value.trim().parse::<i32>().ok())
}

fn parse_timestamp(date: Option<&str>, time: Option<&str>) -> i64 {
    let raw_date = date.unwrap_or("").replace('.', "-");
    let date_part = raw_date.get(..10).unwrap_or(raw_date.as_str());
    let Some(parsed_date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok() else {
        return Utc::now().timestamp();
    };
    let time_part = time
        .and_then(|value| {
            NaiveTime::parse_from_str(value.get(..8).unwrap_or(value), "%H:%M:%S").ok()
        })
        .unwrap_or_default();
    Utc.from_utc_datetime(&parsed_date.and_time(time_part))
        .timestamp()
}

fn normalized_result(value: Option<&str>) -> String {
    match value {
        Some("1-0") | Some("0-1") | Some("1/2-1/2") | Some("*") => value.unwrap_or("*").to_string(),
        _ => "*".to_string(),
    }
}

/// Games against a computer opponent are noise for personal analytics.
pub fn is_bot_game(game: &ParsedGame) -> bool {
    let ai_name = |name: &str| {
        name.trim()
            .to_ascii_lowercase()
            .starts_with("lichess ai level ")
    };
    let computer_name = |name: &str| name.trim().eq_ignore_ascii_case("computer");
    let bot_title = |key: &str| {
        game.tag(key)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("BOT"))
    };
    let white = game.tag("White").unwrap_or("");
    let black = game.tag("Black").unwrap_or("");
    bot_title("WhiteTitle")
        || bot_title("BlackTitle")
        || ai_name(white)
        || ai_name(black)
        || computer_name(white)
        || computer_name(black)
}

/// Builds the domain game model from a parsed PGN.
pub fn build_draft(
    parsed: &ParsedGame,
    source: GameSource,
    external_id: String,
    player_username: &str,
    time_class_override: Option<TimeClass>,
    pgn_override: Option<&str>,
) -> GameDraft {
    let white = parsed.tag("White").unwrap_or("?").to_string();
    let black = parsed.tag("Black").unwrap_or("?").to_string();
    let player = player_username.trim();
    let user_color = if player.is_empty() {
        UserColor::None
    } else if white.eq_ignore_ascii_case(player) {
        UserColor::White
    } else if black.eq_ignore_ascii_case(player) {
        UserColor::Black
    } else {
        UserColor::None
    };
    let time_control = parsed.tag("TimeControl").map(str::to_string);
    let time_class =
        time_class_override.or_else(|| time_class_from_control(time_control.as_deref()));
    let computed = identify_opening(parsed);
    let eco = computed.eco;
    let opening_name = computed.name;
    let opening_ply = computed.ply;
    let opening_source = computed.source;
    let played_at = parse_timestamp(
        parsed.tag("UTCDate").or_else(|| parsed.tag("Date")),
        parsed.tag("UTCTime"),
    );
    let pgn = pgn_override
        .map(str::to_string)
        .unwrap_or_else(|| parsed.to_pgn());
    GameDraft {
        external_id,
        source,
        pgn,
        initial_fen: parsed.initial_fen.clone(),
        white_username: white,
        black_username: black,
        white_elo: parse_elo(&parsed.tags, "WhiteElo"),
        black_elo: parse_elo(&parsed.tags, "BlackElo"),
        user_color,
        result: normalized_result(parsed.tag("Result")),
        termination: parsed.tag("Termination").map(str::to_string),
        time_control,
        time_class,
        eco,
        opening_name,
        opening_ply,
        opening_source,
        played_at,
        total_plies: parsed.moves.len() as i32,
        notes: String::new(),
        is_favorite: false,
        account_id: None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[path = "parser_tests.rs"]
mod tests;
