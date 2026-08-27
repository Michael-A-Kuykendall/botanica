use uuid::Uuid;
use crate::error::DatabaseError;
use crate::types::Cultivar;
use crate::database::BotanicalDatabase;

/// Insert a cultivar
pub async fn insert_cultivar(db: &BotanicalDatabase, cultivar: &Cultivar) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    conn.execute(
        "INSERT INTO cultivars (id, species_id, cultivar_name, trade_name, source, created_at) \
         VALUES (?, ?, ?, ?, ?, current_timestamp)",
        [
            &cultivar.id.to_string() as &dyn duckdb::ToSql,
            &cultivar.species_id.to_string(),
            &cultivar.cultivar_name,
            &cultivar.trade_name,
            &cultivar.source,
        ],
    )?;
    Ok(())
}

/// Get cultivar by id
pub async fn get_cultivar_by_id(db: &BotanicalDatabase, id: Uuid) -> Result<Option<Cultivar>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id, species_id, cultivar_name, trade_name, source FROM cultivars WHERE id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    match stmt.query_row([id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let species_id_str: String = row.get(1)?;
        Ok(Cultivar {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            species_id: Uuid::parse_str(&species_id_str).unwrap_or_default(),
            cultivar_name: row.get(2)?,
            trade_name: row.get(3)?,
            source: row.get(4)?,
        })
    }) {
        Ok(c) => Ok(Some(c)),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::validation(e.to_string())),
    }
}

/// List cultivars for a species
pub async fn get_cultivars_by_species(
    db: &BotanicalDatabase,
    species_id: Uuid,
) -> Result<Vec<Cultivar>, DatabaseError> {
    let conn = db.conn().await;
    let mut stmt = conn
        .prepare(
            "SELECT id, species_id, cultivar_name, trade_name, source FROM cultivars WHERE species_id = ?",
        )
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let rows = stmt
        .query_map([species_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let species_id_str: String = row.get(1)?;
            Ok(Cultivar {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                species_id: Uuid::parse_str(&species_id_str).unwrap_or_default(),
                cultivar_name: row.get(2)?,
                trade_name: row.get(3)?,
                source: row.get(4)?,
            })
        })
        .map_err(|e| DatabaseError::validation(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| DatabaseError::validation(e.to_string()))?);
    }
    Ok(out)
}
