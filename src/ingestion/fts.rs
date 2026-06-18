use sqlx::SqlitePool;
use crate::error::DatabaseError;

/// Populate species_name_fts from existing species + vernacular + synonyms
/// Idempotent: deletes and rebuilds FTS index for fresh snapshot
pub async fn rebuild_species_name_fts(pool: &SqlitePool) -> Result<(), DatabaseError> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM species_name_fts").execute(&mut *tx).await?;

    // Scientific names (species table: genus + specific epithet via join assumed in app layer)
    sqlx::query(
        r#"INSERT INTO species_name_fts (species_id, name, language_code)
           SELECT s.id, (g.name || ' ' || s.specific_epithet), 'la'
           FROM species s JOIN genera g ON s.genus_id = g.id"#,
    )
    .execute(&mut *tx)
    .await?;

    // Synonyms
    sqlx::query(
        r#"INSERT INTO species_name_fts (species_id, name, language_code)
           SELECT species_id, synonym_name, 'la' FROM synonyms"#,
    )
    .execute(&mut *tx)
    .await?;

    // Vernacular names
    sqlx::query(
        r#"INSERT INTO species_name_fts (species_id, name, language_code)
           SELECT species_id, name, COALESCE(language_code,'') FROM vernacular_names"#,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
