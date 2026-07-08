use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::Family;
use crate::database::BotanicalDatabase;

/// Insert a new family into the database
pub async fn insert_family(db: &BotanicalDatabase, family: &Family) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    conn.execute(
        "INSERT INTO families (id, name, authority) VALUES (?, ?, ?)",
        [
            &family.id.to_string() as &dyn duckdb::ToSql,
            &family.name,
            &family.authority,
        ],
    )?;
    Ok(())
}

/// Get a family by ID
pub async fn get_family_by_id(db: &BotanicalDatabase, id: Uuid) -> Result<Option<Family>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare("SELECT id, name, authority FROM families WHERE id = ?")
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let result = stmt.query_row(
        [id.to_string()],
        |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let authority: String = row.get(2)?;
            Ok(Family::with_id(
                Uuid::parse_str(&id_str).unwrap(),
                name,
                authority,
            ))
        },
    );

    match result {
        Ok(family) => Ok(Some(family)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// Get families by name pattern
pub async fn get_families_by_name(db: &BotanicalDatabase, name: &str) -> Result<Vec<Family>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare("SELECT id, name, authority FROM families WHERE name ILIKE ? ORDER BY name")
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let pattern = format!("%{}%", name);
    let rows = stmt
        .query_map([&pattern], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let authority: String = row.get(2)?;
            Ok(Family::with_id(
                Uuid::parse_str(&id_str).unwrap(),
                name,
                authority,
            ))
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut families = Vec::new();
    for row in rows {
        families.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(families)
}

/// Update a family
pub async fn update_family(db: &BotanicalDatabase, id: Uuid, family: &Family) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute(
            "UPDATE families SET name = ?, authority = ? WHERE id = ?",
            [
                &family.name as &dyn duckdb::ToSql,
                &family.authority,
                &id.to_string(),
            ],
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}

/// Delete a family
pub async fn delete_family(db: &BotanicalDatabase, id: Uuid) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute("DELETE FROM families WHERE id = ?", [id.to_string()])
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}
