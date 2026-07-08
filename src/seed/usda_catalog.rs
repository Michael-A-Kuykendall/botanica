//! Bulk load from USDA PlantSearch-derived master CSV.
//! Research-aligned: USDA primary for NA taxonomy foundation (traits via gate scrapes).

use crate::database::BotanicalDatabase;
use crate::error::DatabaseError;
use crate::seed::lookup::parse_scientific_name;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct CatalogStats {
    pub accepted: u32,
    pub skipped_existing: u32,
    pub quarantined: u32,
}

#[derive(Clone)]
struct MasterRow {
    scientific_name: String,
    symbol: String,
    family: String,
    genus: String,
    source: String,
    rank: String,
}

/// High-throughput master CSV ingest (L1 + identifiers + provenance only).
pub async fn ingest_master_csv(
    db: &BotanicalDatabase,
    csv_path: &Path,
) -> Result<CatalogStats, DatabaseError> {
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| DatabaseError::validation(format!("read master csv: {}", e)))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    let mut stats = CatalogStats::default();

    let existing_symbols: std::collections::HashSet<String> = {
        let conn = db.conn().await;
        let mut set = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT external_id FROM species_identifiers WHERE source = 'usda'",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for r in rows.flatten() {
                    set.insert(r);
                }
            }
        }
        set
    };

    let mut batch: Vec<MasterRow> = Vec::new();
    for rec in rdr.deserialize() {
        let row: HashMap<String, String> = rec
            .map_err(|e| DatabaseError::validation(format!("csv row: {}", e)))?;
        let scientific_name = row.get("scientific_name").cloned().unwrap_or_default();
        let symbol = row
            .get("symbol")
            .cloned()
            .or_else(|| row.get("external_id").cloned())
            .unwrap_or_default();
        let family = row.get("family").cloned().unwrap_or_default();
        let genus = row.get("genus").cloned().unwrap_or_default();
        let source = row
            .get("source")
            .cloned()
            .unwrap_or_else(|| "usda".to_string());
        let rank = row
            .get("rank")
            .cloned()
            .unwrap_or_else(|| "species".to_string());

        if symbol.is_empty() || scientific_name.is_empty() || family.is_empty() {
            stats.quarantined += 1;
            continue;
        }
        if existing_symbols.contains(&symbol) {
            stats.skipped_existing += 1;
            continue;
        }
        batch.push(MasterRow {
            scientific_name,
            symbol,
            family,
            genus,
            source,
            rank,
        });
    }

    println!("master rows to insert: {}", batch.len());

    {
        let conn = db.conn().await;
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

        let mut family_ids: HashMap<String, String> = HashMap::new();
        let mut genus_ids: HashMap<String, String> = HashMap::new();

        for (i, row) in batch.iter().enumerate() {
            let (genus_name, epithet, sci) = match parse_scientific_name(&row.scientific_name) {
                Some(p) => p,
                None => {
                    stats.quarantined += 1;
                    continue;
                }
            };
            let genus_name = if row.genus.is_empty() {
                genus_name
            } else {
                row.genus.clone()
            };

            let family_id = ensure_family_sync(&conn, &mut family_ids, &row.family)?;
            let genus_id = ensure_genus_sync(&conn, &mut genus_ids, &family_id, &genus_name)?;

            let species_id = Uuid::new_v4().to_string();
            let rank = row.rank.to_ascii_lowercase();
            let ident_id = Uuid::new_v4().to_string();
            let prov_id = Uuid::new_v4().to_string();

            conn.execute(
                "INSERT INTO species (id, genus_id, specific_epithet, authority, publication_year, \
                 conservation_status, scientific_name, taxonomic_status, rank) \
                 VALUES (?, ?, ?, '', NULL, NULL, ?, 'accepted', ?)",
                [
                    &species_id as &dyn duckdb::ToSql,
                    &genus_id,
                    &epithet,
                    &sci,
                    &rank,
                ],
            )
            .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

            conn.execute(
                "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at) \
                 VALUES (?, ?, ?, ?, 1, current_timestamp)",
                [
                    &ident_id as &dyn duckdb::ToSql,
                    &species_id,
                    &row.source,
                    &row.symbol,
                ],
            )
            .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

            conn.execute(
                "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash) \
                 VALUES (?, ?, 'USDA_PLANTS_CATALOG', ?, 'Public Domain', current_timestamp, NULL)",
                [
                    &prov_id as &dyn duckdb::ToSql,
                    &species_id,
                    &row.symbol,
                ],
            )
            .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

            stats.accepted += 1;
            if i > 0 && i % 5000 == 0 {
                println!("  inserted {} / {}", i, batch.len());
            }
        }

        conn.execute("COMMIT", [])
            .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    }

    Ok(stats)
}

fn ensure_family_sync(
    conn: &duckdb::Connection,
    cache: &mut HashMap<String, String>,
    name: &str,
) -> Result<String, DatabaseError> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    let existing: Result<String, _> =
        conn.query_row("SELECT id FROM families WHERE name = ? LIMIT 1", [name], |r| {
            r.get(0)
        });
    if let Ok(id) = existing {
        cache.insert(name.to_string(), id.clone());
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO families (id, name, authority) VALUES (?, ?, '')",
        [&id as &dyn duckdb::ToSql, &name],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    cache.insert(name.to_string(), id.clone());
    Ok(id)
}

fn ensure_genus_sync(
    conn: &duckdb::Connection,
    cache: &mut HashMap<String, String>,
    family_id: &str,
    name: &str,
) -> Result<String, DatabaseError> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    let existing: Result<String, _> =
        conn.query_row("SELECT id FROM genera WHERE name = ? LIMIT 1", [name], |r| r.get(0));
    if let Ok(id) = existing {
        cache.insert(name.to_string(), id.clone());
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO genera (id, family_id, name, authority) VALUES (?, ?, ?, '')",
        [&id as &dyn duckdb::ToSql, &family_id, &name],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    cache.insert(name.to_string(), id.clone());
    Ok(id)
}
