//! Botanica: cultivated-plant knowledge base on DuckDB.
//!
//! Taxonomy (L1), horticultural knowledge (L2), and inventory schema (L3).
//! See `docs/ARCHITECTURE.md` for layers and product boundaries.

pub mod database;
pub mod types;
pub mod queries;
pub mod migrations;
pub mod error;
pub mod seed;
#[cfg(feature = "ingestion")]
pub mod ingestion;
#[cfg(feature = "ingestion")]
pub mod discovery;

// Optional modules (incomplete unless documented otherwise)
#[cfg(feature = "darwin-core")]
pub mod darwin_core;

#[cfg(feature = "conservation")]
pub mod conservation;

// Re-exports for convenience
pub use database::{BotanicalDatabase, DatabaseConfig};
pub use error::DatabaseError;
pub use types::{Species, Genus, Family, Plant, HealthStatus, Cultivar, SpeciesIdentifier};
pub use migrations::schemas::SCHEMA_VERSION;

/// Result type alias for convenient error handling
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Initialize a new botanical database with migrations
pub async fn initialize_database(database_url: &str) -> Result<BotanicalDatabase> {
    let config = DatabaseConfig::file(database_url);
    let database = BotanicalDatabase::new(config).await?;
    database.migrate().await?;
    Ok(database)
}

/// Create an in-memory database for testing
pub async fn create_test_database() -> Result<BotanicalDatabase> {
    let database = BotanicalDatabase::memory().await?;
    database.migrate().await?;
    Ok(database)
}

// Test modules - only compiled when testing
#[cfg(test)]
mod tests;
