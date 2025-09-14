use sqlx::SqlitePool;
use crate::error::DatabaseError;

/// Search species by scientific name pattern
pub async fn search_species(_pool: &SqlitePool, _query: &str) -> Result<Vec<String>, DatabaseError> {
    // Placeholder implementation - returns empty results
    // In production, this would search species by scientific name patterns
    Ok(Vec::new())
}

/// Search species by common name
pub async fn search_species_by_common_name(_pool: &SqlitePool, _query: &str) -> Result<Vec<String>, DatabaseError> {
    // Placeholder implementation - returns empty results
    // In production, this would search a common_names table or external API
    Ok(vec![])
}

/// Search taxa by keyword across all taxonomic levels
pub async fn search_taxa_by_keyword(_pool: &SqlitePool, _keyword: &str) -> Result<Vec<String>, DatabaseError> {
    // Placeholder implementation - returns empty results
    // In production, this would perform full-text search across taxonomic data
    Ok(vec![])
}