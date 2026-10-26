use crate::db::connection::Db;
use crate::engine::EngineState;

pub struct AppState {
    pub db: Db,
    pub engine: EngineState,
}
