use chenal_lib::db::{analyses_repo, Db};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/suze/Library/Application Support/com.chenal.studio/chenal.db".into());
    let db = Db::open(Path::new(&path))?;
    let updated = analyses_repo::backfill_lichess_accuracy(&db, true)?;
    println!("пересчитано партий: {updated}");
    Ok(())
}
