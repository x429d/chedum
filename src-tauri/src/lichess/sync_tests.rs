use async_trait::async_trait;
use chrono::DateTime;

use super::*;
use crate::db::accounts_repo;
use crate::db::games_repo::list_games;
use crate::domain::game::{GameFilter, GameSource};
use crate::domain::AccountPlatform;
use crate::pgn::parse_single;

const PGN: &str = r#"[Event "Rated rapid game"]
[Site "https://lichess.org/game/abcd1234"]
[Date "2026.10.22"]
[Round "-"]
[White "opponent"]
[Black "chenal_player"]
[Result "1-0"]
[GameId "abcd1234"]
[UTCDate "2026.10.22"]
[UTCTime "17:57:32"]
[WhiteElo "1823"]
[BlackElo "1867"]
[Variant "Standard"]
[TimeControl "600+0"]
[Termination "Normal"]

1. e4 e5 2. Nf3 Nc6 1-0

[Event "Rated chess960 game"]
[Site "https://lichess.org/game/zzzz9999"]
[Date "2026.10.21"]
[Round "-"]
[White "chenal_player"]
[Black "opponent"]
[Result "1-0"]
[GameId "zzzz9999"]
[UTCDate "2026.10.21"]
[UTCTime "17:57:32"]
[Variant "Chess960"]
[TimeControl "600+0"]

1. e4 e5 1-0
"#;

struct MockApi {
    pgn: String,
}

#[async_trait]
impl LichessApi for MockApi {
    async fn user_games_pgn(&self, _username: &str, _since_ms: Option<i64>) -> AppResult<String> {
        Ok(self.pgn.clone())
    }
}

fn account(db: &Db) -> i64 {
    accounts_repo::create(db, AccountPlatform::Lichess, "chenal_player")
        .unwrap()
        .id
}

#[test]
fn parses_since_with_overlap() {
    assert_eq!(since_ms(None), None);
    assert_eq!(since_ms(Some("not a date")), None);
    let value = since_ms(Some("2026-10-22T17:57:32+00:00")).unwrap();
    assert_eq!(
        value,
        DateTime::parse_from_rfc3339("2026-10-22T17:57:32+00:00")
            .unwrap()
            .timestamp_millis()
            - OVERLAP_MS
    );
}

#[test]
fn skips_games_against_lichess_ai() {
    let pgn = "[Event \"casual rapid game\"]\n[Site \"https://lichess.org/game/aiTest01\"]\n[White \"lichess AI level 4\"]\n[Black \"chenal_player\"]\n[Result \"0-1\"]\n[Variant \"Standard\"]\n\n1. e4 e5 0-1";
    let parsed = parse_single(pgn).unwrap();
    assert!(draft_for_game(&parsed, "chenal_player", 1).is_none());
}

#[tokio::test]
async fn sync_imports_standard_games_and_skips_variants() {
    let db = Db::in_memory().unwrap();
    let account_id = account(&db);
    let api = MockApi {
        pgn: PGN.to_string(),
    };
    let result = sync_user(&db, &api, account_id, "chenal_player", None, false, |_| {})
        .await
        .unwrap();
    assert_eq!(result.games_imported, 1);
    assert_eq!(result.games_skipped, 1);
    assert_eq!(result.platform, AccountPlatform::Lichess);

    let page = list_games(&db, &GameFilter::default()).unwrap();
    assert_eq!(page.total, 1);
    let game = &page.games[0];
    assert_eq!(game.source, GameSource::Lichess);
    assert_eq!(game.account_id, Some(account_id));
    assert_eq!(game.account_username.as_deref(), Some("chenal_player"));
    assert_eq!(game.user_color, crate::domain::game::UserColor::Black);

    let again = sync_user(
        &db,
        &api,
        account_id,
        "chenal_player",
        Some("2026-10-22T18:00:00+00:00"),
        false,
        |_| {},
    )
    .await
    .unwrap();
    assert_eq!(again.games_imported, 0);
    assert_eq!(again.games_skipped, 2);
}
