pub mod binary;
pub mod manager;
pub mod pool;
pub mod process;
pub mod pv;
pub mod state;
pub mod types;
pub mod uci;

pub use binary::{bundled_path, discover as discover_binary};
pub use manager::{cancelled_error, is_cancelled_error, EngineManager};
pub use pool::{default_workers, workers_from_setting, EnginePool};
pub use state::EngineState;
pub use types::{EngineLine, EngineUpdate};

/// The development copy of the bundled engine, if it has been fetched.
/// Tests use it and silently skip when it is absent.
pub fn bundled_engine() -> Option<std::path::PathBuf> {
    let path = bundled_path();
    path.is_file().then_some(path)
}
