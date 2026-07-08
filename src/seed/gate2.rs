//! Ingest USDA gate2 normalized JSON (bronze) into L1/L2.

use crate::database::BotanicalDatabase;
use crate::error::DatabaseError;
use crate::seed::lookup::{load_genus_family_map, parse_scientific_name};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Gate2Record {
    scientific_name: Option<String>,
    #[serde(default)]
    common_names: Vec<String>,
    usda_symbol: Option<String>,
    #[serde(default)]
    horticultural_traits: HortTraits,
    #[serde(default)]
    ecological_traits: EcoTraits,
    #[serde(default)]
    distributions: Distributions,
}

#[derive(Debug, Default, Deserialize)]
struct HortTraits {
    #[serde(default)]
    sunlight: Vec<String>,
    #[serde(default)]
    soil: Vec<String>,
    #[serde(default)]
    moisture: Vec<String>,
    #[serde(default)]
    plant_type: Vec<String>,
    #[serde(default)]
    duration: Vec<String>,
    mature_height_cm: Option<f64>,
    toxicity: Option<String>,
    is_cultivated: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct EcoTraits {
    shade_tolerance: Option<String>,
    drought_tolerance: Option<String>,
    salt_tolerance: Option<String>,
    wetland_indicator: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Distributions {
    #[serde(default)]
    native: Vec<String>,
    #[serde(default)]
    introduced: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Gate2Stats {
    pub accepted: u32,
    pub quarantined: u32,
    pub traits: u32,
    pub vernacular: u32,
}

/// Ingest gate2 normalized JSON + genus_family lookup into DB.
pub async fn ingest_gate2_json(
    db: &BotanicalDatabase,
    json_path: &Path,
    genus_family_path: &Path,
) -> Result<Gate2Stats, DatabaseError> {
    let family_map = load_genus_family_map(genus_family_path)?;
    let bytes = std::fs::read(json_path)
        .map_err(|e| DatabaseError::validation(format!("read gate2 json: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let file_hash = hex::encode(hasher.finalize());

    let records: Vec<Gate2Record> = serde_json::from_slice(&bytes)
        .map_err(|e| DatabaseError::validation(format!("parse gate2 json: {}", e)))?;

    let mut stats = Gate2Stats::default();
    // cache family_name -> id, genus_name -> id
    let mut family_ids: HashMap<String, String> = HashMap::new();
    let mut genus_ids: HashMap<String, String> = HashMap::new();

    for rec in records {
        let symbol = rec.usda_symbol.clone().unwrap_or_default();
        let raw_name = rec.scientific_name.clone().unwrap_or_default();

        if symbol.is_empty() {
            quarantine(db, "gate2", "", &raw_name, "missing_symbol").await?;
            stats.quarantined += 1;
            continue;
        }

        // Resolve species_id: reuse if USDA symbol already loaded (enrich path)
        let existing_id: Option<String> = {
            let conn = db.conn().await;
            conn.query_row(
                "SELECT species_id FROM species_identifiers WHERE source = 'usda' AND external_id = ?",
                [&symbol],
                |r| r.get(0),
            )
            .ok()
        };

        let species_id = if let Some(id) = existing_id {
            id
        } else {
            let parsed = match parse_scientific_name(&raw_name) {
                Some(p) => p,
                None => {
                    quarantine(db, "gate2", &symbol, &raw_name, "unparseable_binomial").await?;
                    stats.quarantined += 1;
                    continue;
                }
            };
            let (genus_name, epithet, sci_name) = parsed;

            let family_name = match family_map.get(&genus_name) {
                Some(f) => f.clone(),
                None => {
                    quarantine(
                        db,
                        "gate2",
                        &symbol,
                        &raw_name,
                        "missing_family_lookup",
                    )
                    .await?;
                    stats.quarantined += 1;
                    continue;
                }
            };

            let family_id = ensure_family(db, &mut family_ids, &family_name).await?;
            let genus_id = ensure_genus(db, &mut genus_ids, &family_id, &genus_name).await?;
            let species_id = Uuid::new_v4().to_string();

            {
                let conn = db.conn().await;
                conn.execute(
                    "INSERT INTO species (id, genus_id, specific_epithet, authority, publication_year, \
                     conservation_status, scientific_name, taxonomic_status, rank) \
                     VALUES (?, ?, ?, '', NULL, NULL, ?, 'accepted', 'species')",
                    [
                        &species_id as &dyn duckdb::ToSql,
                        &genus_id,
                        &epithet,
                        &sci_name,
                    ],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

                let ident_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at) \
                     VALUES (?, ?, 'usda', ?, 1, current_timestamp)",
                    [&ident_id as &dyn duckdb::ToSql, &species_id, &symbol],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

                let prov_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash) \
                     VALUES (?, ?, 'USDA_PLANTS_GATE2', ?, 'Public Domain', current_timestamp, ?)",
                    [
                        &prov_id as &dyn duckdb::ToSql,
                        &species_id,
                        &symbol,
                        &file_hash,
                    ],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
            }
            stats.accepted += 1;
            species_id
        };

        // vernacular (enrich even when taxonomy already present)
        for (i, name) in rec.common_names.iter().enumerate() {
            if name.trim().is_empty() {
                continue;
            }
            let id = Uuid::new_v4().to_string();
            let is_primary = if i == 0 { 1i32 } else { 0 };
            let conn = db.conn().await;
            let _ = conn.execute(
                "INSERT INTO vernacular_names (id, species_id, name, language_code, is_primary, source, created_at) \
                 VALUES (?, ?, ?, 'en', ?, 'USDA', current_timestamp)",
                [
                    &id as &dyn duckdb::ToSql,
                    &species_id,
                    name,
                    &is_primary,
                ],
            );
            stats.vernacular += 1;
        }

        // traits / requirements
        stats.traits += insert_text_traits(db, &species_id, &rec).await?;
        insert_distribution(db, &species_id, &rec.distributions).await?;
    }

    Ok(stats)
}

async fn ensure_family(
    db: &BotanicalDatabase,
    cache: &mut HashMap<String, String>,
    name: &str,
) -> Result<String, DatabaseError> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    let conn = db.conn().await;
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

async fn ensure_genus(
    db: &BotanicalDatabase,
    cache: &mut HashMap<String, String>,
    family_id: &str,
    name: &str,
) -> Result<String, DatabaseError> {
    if let Some(id) = cache.get(name) {
        return Ok(id.clone());
    }
    let conn = db.conn().await;
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

async fn quarantine(
    db: &BotanicalDatabase,
    source: &str,
    external_id: &str,
    raw_name: &str,
    reason: &str,
) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO ingest_quarantine (id, source, external_id, raw_name, reason, payload_hash, created_at) \
         VALUES (?, ?, ?, ?, ?, NULL, current_timestamp)",
        [
            &id as &dyn duckdb::ToSql,
            &source,
            &external_id,
            &raw_name,
            &reason,
        ],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(())
}

fn insert_trait_text(
    conn: &duckdb::Connection,
    species_id: &str,
    name: &str,
    value: &str,
) -> Result<bool, DatabaseError> {
    if value.is_empty() {
        return Ok(false);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability, created_at) \
         VALUES (?, ?, ?, ?, NULL, NULL, NULL, 'USDA', 1, current_timestamp)",
        [&id as &dyn duckdb::ToSql, &species_id, &name, &value],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(true)
}

fn insert_req_text(
    conn: &duckdb::Connection,
    species_id: &str,
    req_type: &str,
    value: &str,
) -> Result<bool, DatabaseError> {
    if value.is_empty() {
        return Ok(false);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO cultivation_requirements (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability, created_at) \
         VALUES (?, ?, ?, ?, NULL, NULL, NULL, 'USDA', 1, current_timestamp)",
        [&id as &dyn duckdb::ToSql, &species_id, &req_type, &value],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(true)
}

async fn insert_text_traits(
    db: &BotanicalDatabase,
    species_id: &str,
    rec: &Gate2Record,
) -> Result<u32, DatabaseError> {
    let mut n = 0u32;
    let conn = db.conn().await;

    if !rec.horticultural_traits.plant_type.is_empty() {
        if insert_trait_text(
            &conn,
            species_id,
            "growth_habit",
            &rec.horticultural_traits.plant_type.join("; "),
        )? {
            n += 1;
        }
    }
    if !rec.horticultural_traits.duration.is_empty() {
        if insert_trait_text(
            &conn,
            species_id,
            "duration",
            &rec.horticultural_traits.duration.join("; "),
        )? {
            n += 1;
        }
    }
    if let Some(h) = rec.horticultural_traits.mature_height_cm {
        let id = Uuid::new_v4().to_string();
        let meters = h / 100.0;
        conn.execute(
            "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability, created_at) \
             VALUES (?, ?, 'mature_height', NULL, ?, 'm', NULL, 'USDA', 1, current_timestamp)",
            [&id as &dyn duckdb::ToSql, &species_id, &meters],
        )
        .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        n += 1;
    }
    if let Some(t) = &rec.horticultural_traits.toxicity {
        if insert_trait_text(&conn, species_id, "toxicity", t)? {
            n += 1;
        }
    }
    if !rec.horticultural_traits.sunlight.is_empty() {
        if insert_req_text(
            &conn,
            species_id,
            "sunlight",
            &rec.horticultural_traits.sunlight.join("; "),
        )? {
            n += 1;
        }
    }
    if !rec.horticultural_traits.soil.is_empty() {
        if insert_req_text(
            &conn,
            species_id,
            "soil",
            &rec.horticultural_traits.soil.join("; "),
        )? {
            n += 1;
        }
    }
    if !rec.horticultural_traits.moisture.is_empty() {
        if insert_req_text(
            &conn,
            species_id,
            "moisture",
            &rec.horticultural_traits.moisture.join("; "),
        )? {
            n += 1;
        }
    }
    if let Some(v) = &rec.ecological_traits.drought_tolerance {
        if insert_req_text(&conn, species_id, "drought_tolerance", v)? {
            n += 1;
        }
    }
    if let Some(v) = &rec.ecological_traits.shade_tolerance {
        if insert_req_text(&conn, species_id, "shade_tolerance", v)? {
            n += 1;
        }
    }
    if let Some(v) = &rec.ecological_traits.salt_tolerance {
        if insert_req_text(&conn, species_id, "salinity_tolerance", v)? {
            n += 1;
        }
    }
    if let Some(v) = &rec.ecological_traits.wetland_indicator {
        if insert_req_text(&conn, species_id, "wetland_indicator", v)? {
            n += 1;
        }
    }

    let _ = rec.horticultural_traits.is_cultivated;
    Ok(n)
}

async fn insert_distribution(
    db: &BotanicalDatabase,
    species_id: &str,
    dist: &Distributions,
) -> Result<(), DatabaseError> {
    let conn = db.conn().await;
    for code in dist.native.iter().chain(dist.introduced.iter()) {
        if code.is_empty() {
            continue;
        }
        let id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO distribution_regions (id, species_id, region_code, region_source, notes, source, created_at) \
             VALUES (?, ?, ?, 'USDA', NULL, 'USDA', current_timestamp)",
            [&id as &dyn duckdb::ToSql, &species_id, code],
        );
    }
    Ok(())
}
