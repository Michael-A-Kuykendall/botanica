//! Bulk ingestion: master list → resolved L1 (or quarantine). No Unknown taxa.

use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Bootstrap demo list: scientific_name + external id. Family/genus resolved by parse only
/// when master list is absent — rows without parseable binomial go to quarantine.
pub async fn fetch_cultivated_species_list(
    _http: &reqwest::Client,
    _base_url: &str,
) -> anyhow::Result<Vec<MasterListRow>> {
    let hardcoded = vec![
        MasterListRow {
            scientific_name: "Ocimum basilicum".into(),
            external_id: "demo-ocimum-basilicum".into(),
            family: Some("Lamiaceae".into()),
            genus: Some("Ocimum".into()),
            source: "demo".into(),
        },
        MasterListRow {
            scientific_name: "Petroselinum crispum".into(),
            external_id: "demo-petroselinum-crispum".into(),
            family: Some("Apiaceae".into()),
            genus: Some("Petroselinum".into()),
            source: "demo".into(),
        },
        MasterListRow {
            scientific_name: "Solanum lycopersicum".into(),
            external_id: "demo-solanum-lycopersicum".into(),
            family: Some("Solanaceae".into()),
            genus: Some("Solanum".into()),
            source: "demo".into(),
        },
    ];
    println!("Using demo bootstrap list: {} species (prefer master CSV for real loads)", hardcoded.len());
    Ok(hardcoded)
}

#[derive(Debug, Clone)]
pub struct MasterListRow {
    pub scientific_name: String,
    pub external_id: String,
    pub family: Option<String>,
    pub genus: Option<String>,
    pub source: String,
}

/// Load cultivated species from CSV master list.
/// Expected headers (flexible): scientific_name, symbol|external_id, family, genus, source
pub async fn load_from_master_list(csv_path: &str) -> anyhow::Result<Vec<MasterListRow>> {
    println!("Loading species from master list: {}", csv_path);

    let content = tokio::fs::read_to_string(csv_path).await?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    let mut rows = Vec::new();
    for result in reader.deserialize() {
        let record: std::collections::HashMap<String, String> = result?;
        let sci_name = record
            .get("scientific_name")
            .cloned()
            .or_else(|| record.get("Scientific Name").cloned())
            .unwrap_or_default();
        let external_id = record
            .get("symbol")
            .cloned()
            .or_else(|| record.get("external_id").cloned())
            .or_else(|| record.get("usda_symbol").cloned())
            .unwrap_or_default();
        let family = record.get("family").cloned().filter(|s| !s.is_empty());
        let genus = record.get("genus").cloned().filter(|s| !s.is_empty());
        let source = record
            .get("source")
            .cloned()
            .unwrap_or_else(|| "master_list".to_string());

        if !sci_name.is_empty() && !external_id.is_empty() {
            rows.push(MasterListRow {
                scientific_name: sci_name,
                external_id,
                family,
                genus,
                source,
            });
        }
    }

    println!("Loaded {} species from master list", rows.len());
    Ok(rows)
}

/// Parse "Genus epithet ..." → (genus, epithet)
pub fn parse_binomial(scientific_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = scientific_name.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let genus = parts[0].trim_matches(|c: char| !c.is_alphabetic()).to_string();
    let epithet = parts[1].trim_matches(|c: char| !c.is_alphabetic()).to_string();
    if genus.is_empty() || epithet.is_empty() {
        return None;
    }
    Some((genus, epithet))
}

fn quarantine_insert(
    conn: &duckdb::Connection,
    source: &str,
    external_id: &str,
    raw_name: &str,
    reason: &str,
) -> Result<(), DatabaseError> {
    let id = Uuid::new_v4().to_string();
    let mut hasher = Sha256::new();
    hasher.update(raw_name.as_bytes());
    hasher.update(external_id.as_bytes());
    let hash = hex::encode(hasher.finalize());
    conn.execute(
        "INSERT INTO ingest_quarantine (id, source, external_id, raw_name, reason, payload_hash, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, current_timestamp)",
        [
            &id as &dyn duckdb::ToSql,
            &source,
            &external_id,
            &raw_name,
            &reason,
            &hash,
        ],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(())
}

fn find_or_create_family(conn: &duckdb::Connection, family_name: &str) -> Result<String, DatabaseError> {
    let existing: Result<String, _> = conn.query_row(
        "SELECT id FROM families WHERE name = ? LIMIT 1",
        [family_name],
        |row| row.get(0),
    );
    if let Ok(id) = existing {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO families (id, name, authority) VALUES (?, ?, ?)",
        [&id as &dyn duckdb::ToSql, &family_name, &""],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(id)
}

fn find_or_create_genus(
    conn: &duckdb::Connection,
    family_id: &str,
    genus_name: &str,
) -> Result<String, DatabaseError> {
    let existing: Result<String, _> = conn.query_row(
        "SELECT id FROM genera WHERE name = ? AND family_id = ? LIMIT 1",
        [genus_name, family_id],
        |row| row.get(0),
    );
    if let Ok(id) = existing {
        return Ok(id);
    }
    // Also match genus name alone if already present under any family
    let existing_any: Result<String, _> = conn.query_row(
        "SELECT id FROM genera WHERE name = ? LIMIT 1",
        [genus_name],
        |row| row.get(0),
    );
    if let Ok(id) = existing_any {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO genera (id, family_id, name, authority) VALUES (?, ?, ?, ?)",
        [&id as &dyn duckdb::ToSql, &family_id, &genus_name, &""],
    )
    .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
    Ok(id)
}

/// Ingest master list rows into L1 + identifiers + provenance. Unresolvable → quarantine.
pub async fn bulk_ingest_cultivated(
    db: &BotanicalDatabase,
    max_species: Option<usize>,
    master_list_path: Option<&str>,
) -> Result<(), DatabaseError> {
    let http = reqwest::Client::new();

    let species_list = if let Some(path) = master_list_path {
        load_from_master_list(path)
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to load master list: {}", e)))?
    } else {
        let powo_base = std::env::var("POWO_BASE_URL")
            .unwrap_or_else(|_| "https://powo.science.kew.org/api/2".to_string());
        fetch_cultivated_species_list(&http, &powo_base)
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to fetch species list: {}", e)))?
    };

    let limit = max_species.unwrap_or(species_list.len());
    let to_ingest = &species_list[..std::cmp::min(limit, species_list.len())];

    println!("Starting bulk ingestion of {} species...", to_ingest.len());
    let mut accepted = 0usize;
    let mut quarantined = 0usize;

    for (idx, row) in to_ingest.iter().enumerate() {
        if idx % 10 == 0 {
            println!("[{}/{}] {}", idx, to_ingest.len(), row.scientific_name);
        }

        let err_sci = row.scientific_name.clone();
        let row = row.clone();
        let outcome = db
            .run_in_transaction(move |conn| {
                // Already have external id?
                let existing: Result<String, _> = conn.query_row(
                    "SELECT species_id FROM species_identifiers WHERE source = ? AND external_id = ?",
                    [&row.source, &row.external_id],
                    |r| r.get(0),
                );
                if existing.is_ok() {
                    return Ok(());
                }

                let (genus_name, epithet) = match parse_binomial(&row.scientific_name) {
                    Some(p) => p,
                    None => {
                        quarantine_insert(
                            conn,
                            &row.source,
                            &row.external_id,
                            &row.scientific_name,
                            "unparseable_binomial",
                        )?;
                        return Ok(());
                    }
                };

                let genus_name = row.genus.clone().unwrap_or(genus_name);
                let family_name = match row.family.clone() {
                    Some(f) if !f.is_empty() => f,
                    _ => {
                        quarantine_insert(
                            conn,
                            &row.source,
                            &row.external_id,
                            &row.scientific_name,
                            "missing_family",
                        )?;
                        return Ok(());
                    }
                };

                let family_id = find_or_create_family(conn, &family_name)?;
                let genus_id = find_or_create_genus(conn, &family_id, &genus_name)?;
                let species_id = Uuid::new_v4().to_string();

                conn.execute(
                    "INSERT INTO species (id, genus_id, specific_epithet, authority, publication_year, \
                     conservation_status, scientific_name, taxonomic_status, rank) \
                     VALUES (?, ?, ?, '', NULL, NULL, ?, 'accepted', 'species')",
                    [
                        &species_id as &dyn duckdb::ToSql,
                        &genus_id,
                        &epithet,
                        &row.scientific_name,
                    ],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

                let ident_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at) \
                     VALUES (?, ?, ?, ?, 1, current_timestamp)",
                    [
                        &ident_id as &dyn duckdb::ToSql,
                        &species_id,
                        &row.source,
                        &row.external_id,
                    ],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

                let prov_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash) \
                     VALUES (?, ?, ?, ?, 'see source', current_timestamp, NULL)",
                    [
                        &prov_id as &dyn duckdb::ToSql,
                        &species_id,
                        &row.source,
                        &row.external_id,
                    ],
                )
                .map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

                Ok(())
            })
            .await;

        match outcome {
            Ok(()) => {
                // rough: we don't know quarantine vs accept inside without return value —
                // count via separate check is heavy; increment accepted for ok path only
                accepted += 1;
            }
            Err(e) => {
                eprintln!("error on {}: {}", err_sci, e);
                quarantined += 1;
            }
        }
    }

    // Report quarantine size
    {
        let conn = db.conn().await;
        let q: i64 = conn
            .query_row("SELECT COUNT(*) FROM ingest_quarantine", [], |r| r.get(0))
            .unwrap_or(0);
        println!(
            "Bulk done. transactions_ok={} quarantine_rows={}",
            accepted, q
        );
        let _ = quarantined;
    }

    Ok(())
}
