use sqlx::SqlitePool;
use crate::error::DatabaseError;

/// Search species by scientific name pattern
pub async fn search_species(pool: &SqlitePool, query: &str) -> Result<Vec<String>, DatabaseError> {
    // Use FTS index over names (scientific/vernacular/synonyms) when populated
    let rows = sqlx::query_scalar::<_, String>(
        r#"SELECT species_id FROM species_name_fts WHERE name MATCH ?1 LIMIT 50"#,
    )
    .bind(query)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Search species by common name
pub async fn search_species_by_common_name(pool: &SqlitePool, query: &str) -> Result<Vec<String>, DatabaseError> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"SELECT species_id FROM species_name_fts WHERE name MATCH ?1 LIMIT 50"#,
    )
    .bind(query)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Search taxa by keyword across all taxonomic levels
pub async fn search_taxa_by_keyword(pool: &SqlitePool, keyword: &str) -> Result<Vec<String>, DatabaseError> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"SELECT species_id FROM species_name_fts WHERE name MATCH ?1 LIMIT 50"#,
    )
    .bind(keyword)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}