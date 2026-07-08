use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::Species;
use crate::database::BotanicalDatabase;

fn map_species_row(row: &duckdb::Row<'_>) -> duckdb::Result<Species> {
    let id_str: String = row.get(0)?;
    let genus_id_str: String = row.get(1)?;
    let specific_epithet: String = row.get(2)?;
    let authority: String = row.get(3)?;
    let publication_year: Option<i32> = row.get(4)?;
    let conservation_status: Option<String> = row.get(5)?;
    let scientific_name: Option<String> = row.get(6)?;
    let taxonomic_status: Option<String> = row.get(7)?;
    let rank: Option<String> = row.get(8)?;
    Ok(Species::with_id(
        Uuid::parse_str(&id_str).unwrap_or_default(),
        Uuid::parse_str(&genus_id_str).unwrap_or_default(),
        specific_epithet,
        authority,
        publication_year,
        conservation_status,
    )
    .with_taxonomy(
        scientific_name,
        taxonomic_status.unwrap_or_else(|| "accepted".into()),
        rank.unwrap_or_else(|| "species".into()),
    ))
}

const SPECIES_SELECT: &str = "SELECT id, genus_id, specific_epithet, authority, publication_year, \
    conservation_status, scientific_name, taxonomic_status, rank FROM species";

/// Insert a new species into the database
pub async fn insert_species(db: &BotanicalDatabase, species: &Species) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    conn.execute(
        "INSERT INTO species (id, genus_id, specific_epithet, authority, publication_year, \
         conservation_status, scientific_name, taxonomic_status, rank) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        [
            &species.id.to_string() as &dyn duckdb::ToSql,
            &species.genus_id.to_string(),
            &species.specific_epithet,
            &species.authority,
            &species.publication_year,
            &species.conservation_status,
            &species.scientific_name,
            &species.taxonomic_status,
            &species.rank,
        ],
    )?;
    Ok(())
}

/// Get a species by ID
pub async fn get_species_by_id(db: &BotanicalDatabase, id: Uuid) -> Result<Option<Species>, DatabaseError> {
    let conn = db.conn().await;
    let sql = format!("{} WHERE id = ?", SPECIES_SELECT);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let result = stmt.query_row([id.to_string()], map_species_row);

    match result {
        Ok(species) => Ok(Some(species)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// Get species by epithet or scientific_name pattern
pub async fn get_species_by_name(db: &BotanicalDatabase, name: &str) -> Result<Vec<Species>, DatabaseError> {
    let conn = db.conn().await;
    let sql = format!(
        "{} WHERE specific_epithet ILIKE ? OR scientific_name ILIKE ?",
        SPECIES_SELECT
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let pattern = format!("%{}%", name);
    let rows = stmt
        .query_map([&pattern, &pattern], map_species_row)
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut species = Vec::new();
    for row in rows {
        species.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(species)
}

/// Update a species
pub async fn update_species(db: &BotanicalDatabase, id: Uuid, species: &Species) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute(
            "UPDATE species SET genus_id = ?, specific_epithet = ?, authority = ?, \
             publication_year = ?, conservation_status = ?, scientific_name = ?, \
             taxonomic_status = ?, rank = ? WHERE id = ?",
            [
                &species.genus_id.to_string() as &dyn duckdb::ToSql,
                &species.specific_epithet,
                &species.authority,
                &species.publication_year,
                &species.conservation_status,
                &species.scientific_name,
                &species.taxonomic_status,
                &species.rank,
                &id.to_string(),
            ],
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}

/// Delete a species
pub async fn delete_species(db: &BotanicalDatabase, id: Uuid) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute("DELETE FROM species WHERE id = ?", [id.to_string()])
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}
