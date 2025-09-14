//! Botanica: Professional botanical cultivation database with AI-powered plant insights
//! 
//! This crate provides type-safe botanical data management with taxonomic validation,
//! geospatial operations, and scientific nomenclature handling.

pub mod database;
pub mod types;
pub mod queries;
pub mod migrations;
pub mod error;

#[cfg(feature = "contextlite")]
pub mod contextlite;

// Professional features (v0.2.0+) - optional and additive only
#[cfg(feature = "darwin-core")]
pub mod darwin_core;

#[cfg(feature = "conservation")]
pub mod conservation;

// Re-exports for convenience
pub use database::{BotanicalDatabase, DatabaseConfig};
pub use error::DatabaseError;
pub use types::{Species, Genus, Family};

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
