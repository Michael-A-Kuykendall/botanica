use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::SpeciesIdentifier;
use crate::database::BotanicalDatabase;

/// Insert external identifier (idempotent on source+external_id unique)
pub async fn insert_identifier(
    db: &BotanicalDatabase,
    ident: &SpeciesIdentifier,
) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    conn.execute(
        "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at) \
         VALUES (?, ?, ?, ?, ?, current_timestamp) \
         ON CONFLICT (source, external_id) DO NOTHING",
        [
            &ident.id.to_string() as &dyn duckdb::ToSql,
            &ident.species_id.to_string(),
            &ident.source,
            &ident.external_id,
            &(if ident.is_primary { 1i32 } else { 0i32 }),
        ],
    )?;
    Ok(())
}

/// Resolve species_id by external source key
pub async fn find_species_by_external_id(
    db: &BotanicalDatabase,
    source: &str,
    external_id: &str,
) -> Result<Option<Uuid>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT species_id FROM species_identifiers WHERE source = ? AND external_id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    match stmt.query_row([source, external_id], |row| {
        let s: String = row.get(0)?;
        Ok(s)
    }) {
        Ok(s) => Ok(Uuid::parse_str(&s).ok()),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// List identifiers for a species
pub async fn get_identifiers_for_species(
    db: &BotanicalDatabase,
    species_id: Uuid,
) -> Result<Vec<SpeciesIdentifier>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id, species_id, source, external_id, is_primary FROM species_identifiers WHERE species_id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let rows = stmt
        .query_map([species_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let sid: String = row.get(1)?;
            let is_primary: i32 = row.get(4)?;
            Ok(SpeciesIdentifier {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                species_id: Uuid::parse_str(&sid).unwrap_or_default(),
                source: row.get(2)?,
                external_id: row.get(3)?,
                is_primary: is_primary != 0,
            })
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(out)
}
