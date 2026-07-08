use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;

pub mod runner;
pub mod schemas;

/// Run all database migrations
pub async fn run_migrations(db: &BotanicalDatabase) -> Result<(), DatabaseError> {
    runner::run_migrations(db).await
}

pub use schemas::SCHEMA_VERSION;
pub use runner::{validate_migrations, get_migration_status, check_schema_version};
