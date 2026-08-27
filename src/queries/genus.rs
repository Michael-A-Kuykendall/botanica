use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::Genus;
use crate::database::BotanicalDatabase;

/// Insert a new genus into the database
pub async fn insert_genus(db: &BotanicalDatabase, genus: &Genus) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    conn.execute(
        "INSERT INTO genera (id, family_id, name, authority) VALUES (?, ?, ?, ?)",
        [
            &genus.id.to_string() as &dyn duckdb::ToSql,
            &genus.family_id.to_string(),
            &genus.name,
            &genus.authority,
        ],
    )?;
    Ok(())
}

/// Get a genus by ID
pub async fn get_genus_by_id(db: &BotanicalDatabase, id: Uuid) -> Result<Option<Genus>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare("SELECT id, family_id, name, authority FROM genera WHERE id = ?")
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let result = stmt.query_row(
        [id.to_string()],
        |row| {
            let id_str: String = row.get(0)?;
            let family_id_str: String = row.get(1)?;
            let name: String = row.get(2)?;
            let authority: String = row.get(3)?;
            Ok(Genus::with_id(
                Uuid::parse_str(&id_str).unwrap(),
                Uuid::parse_str(&family_id_str).unwrap(),
                name,
                authority,
            ))
        },
    );

    match result {
        Ok(genus) => Ok(Some(genus)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// Get genera by family ID
pub async fn get_genera_by_family_id(db: &BotanicalDatabase, family_id: Uuid) -> Result<Vec<Genus>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare("SELECT id, family_id, name, authority FROM genera WHERE family_id = ? ORDER BY name")
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let rows = stmt
        .query_map([family_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let family_id_str: String = row.get(1)?;
            let name: String = row.get(2)?;
            let authority: String = row.get(3)?;
            Ok(Genus::with_id(
                Uuid::parse_str(&id_str).unwrap(),
                Uuid::parse_str(&family_id_str).unwrap(),
                name,
                authority,
            ))
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut genera = Vec::new();
    for row in rows {
        genera.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(genera)
}

/// Update a genus
pub async fn update_genus(db: &BotanicalDatabase, id: Uuid, genus: &Genus) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute(
            "UPDATE genera SET family_id = ?, name = ?, authority = ? WHERE id = ?",
            [
                &genus.family_id.to_string() as &dyn duckdb::ToSql,
                &genus.name,
                &genus.authority,
                &id.to_string(),
            ],
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}

/// Delete a genus
pub async fn delete_genus(db: &BotanicalDatabase, id: Uuid) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute("DELETE FROM genera WHERE id = ?", [id.to_string()])
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}
