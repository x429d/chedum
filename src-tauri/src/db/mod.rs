pub mod accounts_repo;
pub mod analyses_repo;
pub mod archives_repo;
pub mod connection;
pub mod games_delete;
pub mod games_repo;
pub mod migrations;
pub mod schema;
pub mod seed;
pub mod settings_repo;
pub mod stats_extra;
pub mod stats_repo;

pub use connection::Db;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;
