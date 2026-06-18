use sqlx::SqlitePool;
use crate::error::DatabaseError;
use super::schemas::{
    FAMILIES_TABLE_SQL, GENERA_TABLE_SQL, SPECIES_TABLE_SQL,
    PLANTS_TABLE_SQL, PLANT_PHOTOS_TABLE_SQL, CARE_ACTIVITIES_TABLE_SQL,
    ENVIRONMENTS_TABLE_SQL, CULTIVATION_RECORDS_TABLE_SQL,
    SYNONYMS_TABLE_SQL, VERNACULAR_NAMES_TABLE_SQL, DISTRIBUTION_REGIONS_TABLE_SQL,
    TRAITS_TABLE_SQL, SEASONAL_CHARACTERISTICS_TABLE_SQL, CULTIVATION_REQUIREMENTS_TABLE_SQL,
    ECOLOGICAL_INTERACTIONS_TABLE_SQL, USES_TABLE_SQL, MEDIA_TABLE_SQL, PROVENANCE_TABLE_SQL,
    SPECIES_NAME_FTS_SQL
};

/// Run all database migrations
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DatabaseError> {
    // Create taxonomy tables (reference data)
    sqlx::query(FAMILIES_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(GENERA_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(SPECIES_TABLE_SQL)
        .execute(pool)
        .await?;

    // Create horticultural reference tables
    sqlx::query(SYNONYMS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(VERNACULAR_NAMES_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(DISTRIBUTION_REGIONS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(TRAITS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(SEASONAL_CHARACTERISTICS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(CULTIVATION_REQUIREMENTS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(ECOLOGICAL_INTERACTIONS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(USES_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(MEDIA_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(PROVENANCE_TABLE_SQL)
        .execute(pool)
        .await?;

    // Create user cultivation tables (application data)
    sqlx::query(PLANTS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(PLANT_PHOTOS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(CARE_ACTIVITIES_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(ENVIRONMENTS_TABLE_SQL)
        .execute(pool)
        .await?;

    sqlx::query(CULTIVATION_RECORDS_TABLE_SQL)
        .execute(pool)
        .await?;

    // Create FTS virtual table for names
    sqlx::query(SPECIES_NAME_FTS_SQL)
        .execute(pool)
        .await?;

    Ok(())
}

/// Validate that all required migrations have been applied
pub async fn validate_migrations(_pool: &SqlitePool) -> Result<bool, DatabaseError> {
    // Placeholder implementation - in production would check migration table
    // and verify all expected migrations are present and applied successfully
    Ok(true)
}

/// Get the current migration status
pub async fn get_migration_status(_pool: &SqlitePool) -> Result<Vec<String>, DatabaseError> {
    // Placeholder implementation - returns empty migration status
    // In production, this would query the migrations table and return
    // a list of applied migrations with timestamps
    Ok(vec![])
}

/// Check if database schema is up to date
pub async fn check_schema_version(_pool: &SqlitePool) -> Result<String, DatabaseError> {
    // Placeholder implementation - returns current version
    // In production, this would check the actual schema version
    Ok("0.2.0".to_string())
}