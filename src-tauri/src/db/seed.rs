use serde::Deserialize;

use crate::db::analyses_repo::insert_analysis_conn;
use crate::db::connection::Db;
use crate::db::games_repo::{count_games, insert_game};
use crate::domain::analysis::{AnalysisDraft, GameAnalysis, MoveEvaluation};
use crate::domain::error::AppResult;
use crate::domain::game::GameDraft;

static SEED_JSON: &str = include_str!("../../resources/demo_seed.json");

#[derive(Debug, Deserialize)]
struct SeedEntry {
    game: GameDraft,
    analysis: GameAnalysis,
    moves: Vec<MoveEvaluation>,
}

/// Заполняет пустую базу демонстрационными партиями с готовым анализом.
pub fn ensure_demo(db: &Db) -> AppResult<u32> {
    if count_games(db)? > 0 {
        return Ok(0);
    }
    let entries: Vec<SeedEntry> = serde_json::from_str(SEED_JSON)?;
    db.with_tx(|tx| {
        let mut imported = 0;
        for entry in entries {
            let Some(game_id) = insert_game(tx, &entry.game)? else {
                continue;
            };
            let mut analysis = entry.analysis;
            analysis.game_id = game_id;
            let mut moves = entry.moves;
            for mv in &mut moves {
                mv.game_id = game_id;
            }
            insert_analysis_conn(tx, &AnalysisDraft { analysis, moves })?;
            imported += 1;
        }
        Ok(imported)
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::db::analyses_repo::get_analysis;
    use crate::db::games_repo::{count_games, list_games};
    use crate::domain::game::GameFilter;

    #[test]
    fn demo_seed_loads_four_analyzed_games() {
        let db = Db::in_memory().unwrap();
        let imported = ensure_demo(&db).unwrap();
        assert_eq!(imported, 4);
        assert_eq!(count_games(&db).unwrap(), 4);
        let page = list_games(&db, &GameFilter::default()).unwrap();
        assert!(page.games.iter().all(|game| game.has_analysis));
        let first = page.games.first().unwrap();
        let analysis = get_analysis(&db, first.id).unwrap().unwrap();
        assert!(analysis.white_accuracy.is_some());
        assert!(analysis.summary_text.len() > 20);
        assert_eq!(ensure_demo(&db).unwrap(), 0);
    }
}
