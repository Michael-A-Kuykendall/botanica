use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::{Plant, HealthStatus};
use crate::database::BotanicalDatabase;

fn map_plant_row(row: &duckdb::Row<'_>) -> duckdb::Result<Plant> {
    let id_str: String = row.get(0)?;
    let species_id_str: Option<String> = row.get(1)?;
    let cultivar_id_str: Option<String> = row.get(2)?;
    let user_given_name: String = row.get(3)?;
    let health: String = row.get(4).unwrap_or_else(|_| "unknown".to_string());
    let acquired_date: Option<String> = row.get(5)?;
    let location: Option<String> = row.get(6)?;
    let notes: Option<String> = row.get(7)?;
    let user_id: Option<String> = row.get(8)?;
    let device_id: Option<String> = row.get(9)?;

    Ok(Plant {
        id: Uuid::parse_str(&id_str).unwrap_or_default(),
        species_id: species_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
        cultivar_id: cultivar_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
        user_given_name,
        health_status: HealthStatus::parse(&health),
        acquired_date,
        location,
        notes,
        user_id,
        device_id,
    })
}

/// Insert a plant (L3 inventory)
pub async fn insert_plant(db: &BotanicalDatabase, plant: &Plant) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    let species_id = plant.species_id.map(|u| u.to_string());
    let cultivar_id = plant.cultivar_id.map(|u| u.to_string());
    let health = plant.health_status.as_str().to_string();
    let id = plant.id.to_string();
    conn.execute(
        "INSERT INTO plants (id, species_id, cultivar_id, user_given_name, health_status, \
         acquired_date, location, notes, user_id, device_id, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, current_timestamp, current_timestamp)",
        [
            &id as &dyn duckdb::ToSql,
            &species_id,
            &cultivar_id,
            &plant.user_given_name,
            &health,
            &plant.acquired_date,
            &plant.location,
            &plant.notes,
            &plant.user_id,
            &plant.device_id,
        ],
    )?;
    Ok(())
}

/// Get plant by id
pub async fn get_plant_by_id(db: &BotanicalDatabase, id: Uuid) -> Result<Option<Plant>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id, species_id, cultivar_id, user_given_name, health_status, \
             acquired_date, location, notes, user_id, device_id FROM plants WHERE id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    match stmt.query_row([id.to_string()], map_plant_row) {
        Ok(p) => Ok(Some(p)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// List plants for a species
pub async fn get_plants_by_species(db: &BotanicalDatabase, species_id: Uuid) -> Result<Vec<Plant>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id, species_id, cultivar_id, user_given_name, health_status, \
             acquired_date, location, notes, user_id, device_id FROM plants WHERE species_id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let rows = stmt
        .query_map([species_id.to_string()], map_plant_row)
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(out)
}

/// Update plant fields + bump updated_at (sync-friendly)
pub async fn update_plant(db: &BotanicalDatabase, plant: &Plant) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let species_id = plant.species_id.map(|u| u.to_string());
    let cultivar_id = plant.cultivar_id.map(|u| u.to_string());
    let health = plant.health_status.as_str().to_string();
    let id = plant.id.to_string();
    let affected = conn
        .execute(
            "UPDATE plants SET species_id = ?, cultivar_id = ?, user_given_name = ?, \
             health_status = ?, acquired_date = ?, location = ?, notes = ?, \
             user_id = ?, device_id = ?, updated_at = current_timestamp WHERE id = ?",
            [
                &species_id as &dyn duckdb::ToSql,
                &cultivar_id,
                &plant.user_given_name,
                &health,
                &plant.acquired_date,
                &plant.location,
                &plant.notes,
                &plant.user_id,
                &plant.device_id,
                &id,
            ],
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}

/// Delete a plant
pub async fn delete_plant(db: &BotanicalDatabase, id: Uuid) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    let affected = conn
        .execute("DELETE FROM plants WHERE id = ?", [id.to_string()])
        .map_err(|e| DatabaseError::validation(e.to_string()))?;
    Ok(affected > 0)
}
