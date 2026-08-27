use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;

/// Search species by epithet or scientific_name pattern
pub async fn search_species(db: &BotanicalDatabase, query: &str) -> Result<Vec<String>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM species \
             WHERE specific_epithet ILIKE ? OR scientific_name ILIKE ? \
             LIMIT 50",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let pattern = format!("%{}%", query);
    let rows = stmt
        .query_map([&pattern, &pattern], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(results)
}

/// Search species by common name
pub async fn search_species_by_common_name(db: &BotanicalDatabase, query: &str) -> Result<Vec<String>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare("SELECT species_id FROM vernacular_names WHERE name ILIKE ? LIMIT 50")
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let pattern = format!("%{}%", query);
    let rows = stmt
        .query_map([&pattern], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(results)
}

/// Search taxa by keyword across all taxonomic levels
pub async fn search_taxa_by_keyword(db: &BotanicalDatabase, keyword: &str) -> Result<Vec<String>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT s.id FROM species s
             LEFT JOIN genera g ON s.genus_id = g.id
             LEFT JOIN families f ON g.family_id = f.id
             LEFT JOIN vernacular_names vn ON s.id = vn.species_id
             WHERE s.specific_epithet ILIKE ?
                OR s.scientific_name ILIKE ?
                OR g.name ILIKE ?
                OR f.name ILIKE ?
                OR vn.name ILIKE ?
             LIMIT 50"
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let pattern = format!("%{}%", keyword);
    let rows = stmt
        .query_map(
            [&pattern, &pattern, &pattern, &pattern, &pattern],
            |row| {
                let id: String = row.get(0)?;
                Ok(id)
            },
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(results)
}
