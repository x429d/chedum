use std::path::Path;

use chedum_lib::db::{analyses_repo, Db};

fn default_db_path() -> String {
    chedum_lib::paths::database_path().display().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(default_db_path);
    let db = Db::open(Path::new(&path))?;
    let updated = analyses_repo::backfill_lichess_accuracy(&db, true)?;
    println!("recalculated analyses: {updated}");
    Ok(())
}
