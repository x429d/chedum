use async_trait::async_trait;

use super::*;
use crate::chesscom::client::RemoteGame;
use crate::db::accounts_repo;
use crate::db::connection::Db;
use crate::db::games_repo::count_games;
use crate::domain::AccountPlatform;

const LEGAL: &str = include_str!("../../resources/demo_legal.pgn");
const ARCHIVE: &str = "https://api.chess.com/pub/player/chenal_player/games/2020/05";

fn account(db: &Db) -> i64 {
    accounts_repo::create(db, AccountPlatform::Chesscom, "chenal_player")
        .unwrap()
        .id
}

struct MockApi {
    archives: Vec<String>,
    games: Vec<RemoteGame>,
}

#[async_trait]
impl ChessComApi for MockApi {
    async fn archives(&self, _username: &str) -> AppResult<Vec<String>> {
        Ok(self.archives.clone())
    }

    async fn archive_games(&self, _archive_url: &str) -> AppResult<Vec<RemoteGame>> {
        Ok(self.games.clone())
    }
}

fn mock() -> MockApi {
    MockApi {
        archives: vec![ARCHIVE.to_string()],
        games: vec![RemoteGame {
            url: "https://www.chess.com/game/live/42".to_string(),
            pgn: LEGAL.to_string(),
            time_control: "300+0".to_string(),
            end_time: 1_600_000_000,
            time_class: "blitz".to_string(),
            rules: "chess".to_string(),
            ..RemoteGame::default()
        }],
    }
}

#[test]
fn parses_archive_urls() {
    assert_eq!(parse_archive_url(ARCHIVE), Some((2020, 5)));
    assert_eq!(
        parse_archive_url("https://api.chess.com/pub/player/x/games/2024/12/"),
        Some((2024, 12))
    );
    assert_eq!(
        parse_archive_url("https://api.chess.com/pub/player/x"),
        None
    );
    assert_eq!(
        parse_archive_url("https://api.chess.com/pub/player/x/games/2024/13"),
        None
    );
}

#[test]
fn incremental_plan_skips_finished_archives() {
    let existing = vec![ArchiveRecord {
        username: "chenal_player".to_string(),
        archive_url: ARCHIVE.to_string(),
        year: 2020,
        month: 5,
        is_fully_synced: true,
        last_synced_at: "2020-06-01T00:00:00Z".to_string(),
    }];
    let archives = vec![
        ARCHIVE.to_string(),
        "https://api.chess.com/pub/player/chenal_player/games/2026/10".to_string(),
    ];
    let plan = plan_archives(&existing, &archives, 2026, 10, false);
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].year, 2026);
    let forced = plan_archives(&existing, &archives, 2026, 10, true);
    assert_eq!(forced.len(), 2);
    let current = plan_archives(&existing, &[ARCHIVE.to_string()], 2020, 5, false);
    assert_eq!(current.len(), 1);
}

#[tokio::test]
async fn sync_imports_then_dedupes() {
    let db = Db::in_memory().unwrap();
    let account_id = account(&db);
    let api = mock();
    let mut events = 0u32;
    let first = sync_user(&db, &api, account_id, "chenal_player", false, |_| {
        events += 1
    })
    .await
    .unwrap();
    assert_eq!(first.games_imported, 1);
    assert_eq!(first.archives_processed, 1);
    assert_eq!(first.account_id, account_id);
    assert_eq!(count_games(&db).unwrap(), 1);
    assert!(events > 0);

    let second = sync_user(&db, &api, account_id, "chenal_player", false, |_| {})
        .await
        .unwrap();
    assert_eq!(second.archives_processed, 0);
    assert_eq!(second.games_imported, 0);

    let forced = sync_user(&db, &api, account_id, "chenal_player", true, |_| {})
        .await
        .unwrap();
    assert_eq!(forced.archives_processed, 1);
    assert_eq!(forced.games_imported, 0);
    assert_eq!(forced.games_skipped, 1);
    assert_eq!(count_games(&db).unwrap(), 1);
}

#[tokio::test]
async fn sync_rejects_empty_username() {
    let db = Db::in_memory().unwrap();
    let account_id = account(&db);
    let api = mock();
    let error = sync_user(&db, &api, account_id, "  ", false, |_| {})
        .await
        .unwrap_err();
    assert!(matches!(error, AppError::Validation(_)));
}
