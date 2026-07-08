use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;

/// Populate full-text search index from existing species + vernacular + synonyms
/// Uses DuckDB's ILIKE for search (simple approach, can upgrade to FTS extension later)
pub async fn rebuild_species_name_fts(_db: &BotanicalDatabase) -> Result<(), DatabaseError> {
    // DuckDB doesn't have SQLite-style FTS5 virtual tables.
    // Search is handled via ILIKE queries in the search module.
    // This function is kept for API compatibility.
    Ok(())
}
