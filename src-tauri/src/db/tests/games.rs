use super::support::draft;
use crate::db::accounts_repo;
use crate::db::connection::Db;
use crate::db::games_repo::{count_games, get_game, insert_game, list_games};
use crate::domain::game::{GameFilter, GameSource, TimeClass, UserColor};
use crate::domain::AccountPlatform;

#[test]
fn inserts_dedupes_and_filters_games() {
    let db = Db::in_memory().unwrap();
    let first = draft(
        "g1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    let second = draft(
        "g2",
        "bob",
        "alice",
        UserColor::Black,
        "1-0",
        TimeClass::Blitz,
        200,
    );
    let id1 = db.with_tx(|tx| insert_game(tx, &first)).unwrap().unwrap();
    let id2 = db.with_tx(|tx| insert_game(tx, &second)).unwrap().unwrap();
    assert!(db.with_tx(|tx| insert_game(tx, &first)).unwrap().is_none());
    assert_eq!(count_games(&db).unwrap(), 2);

    let all = list_games(&db, &GameFilter::default()).unwrap();
    assert_eq!(all.total, 2);
    assert_eq!(all.games[0].id, id2);

    let white = list_games(
        &db,
        &GameFilter {
            color: Some(UserColor::White),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(white.games.len(), 1);
    assert_eq!(white.games[0].id, id1);

    let blitz = list_games(
        &db,
        &GameFilter {
            time_class: Some(TimeClass::Blitz),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(blitz.games.len(), 1);

    let wins = list_games(
        &db,
        &GameFilter {
            result: Some("win".to_string()),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(wins.games.len(), 1);
    assert_eq!(wins.games[0].id, id1);

    let game = get_game(&db, id1).unwrap();
    assert_eq!(game.white_username, "alice");
    assert_eq!(game.user_result(), Some("win"));
}

#[test]
fn account_filter_and_source_are_visible_in_the_library() {
    let db = Db::in_memory().unwrap();
    let chess = accounts_repo::create(&db, AccountPlatform::Chesscom, "alice").unwrap();
    let lichess = accounts_repo::create(&db, AccountPlatform::Lichess, "alice_li").unwrap();
    let mut first = draft(
        "g-account-1",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        100,
    );
    first.source = GameSource::Chesscom;
    first.account_id = Some(chess.id);
    let mut second = draft(
        "g-account-2",
        "bob",
        "alice_li",
        UserColor::Black,
        "1-0",
        TimeClass::Blitz,
        200,
    );
    second.source = GameSource::Lichess;
    second.account_id = Some(lichess.id);
    db.with_tx(|tx| insert_game(tx, &first)).unwrap().unwrap();
    db.with_tx(|tx| insert_game(tx, &second)).unwrap().unwrap();

    let page = list_games(
        &db,
        &GameFilter {
            account_id: Some(lichess.id),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(page.total, 1);
    let game = &page.games[0];
    assert_eq!(game.account_id, Some(lichess.id));
    assert_eq!(game.account_username.as_deref(), Some("alice_li"));
    assert_eq!(game.account_platform, Some(AccountPlatform::Lichess));
    assert_eq!(game.source, GameSource::Lichess);

    let all = list_games(&db, &GameFilter::default()).unwrap();
    assert_eq!(all.total, 2);
}

#[test]
fn date_filters_limit_the_library() {
    let db = Db::in_memory().unwrap();
    let early = draft(
        "g-early",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        1_672_531_200,
    );
    let late = draft(
        "g-late",
        "alice",
        "bob",
        UserColor::White,
        "1-0",
        TimeClass::Rapid,
        1_688_169_600,
    );
    db.with_tx(|tx| insert_game(tx, &early)).unwrap().unwrap();
    db.with_tx(|tx| insert_game(tx, &late)).unwrap().unwrap();

    let from = list_games(
        &db,
        &GameFilter {
            date_from: Some(1_680_000_000),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(from.games.len(), 1);
    assert_eq!(from.games[0].played_at, 1_688_169_600);

    let to = list_games(
        &db,
        &GameFilter {
            date_to: Some(1_680_000_000),
            ..GameFilter::default()
        },
    )
    .unwrap();
    assert_eq!(to.games.len(), 1);
    assert_eq!(to.games[0].played_at, 1_672_531_200);
}
