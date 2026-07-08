use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use super::normalization::TraitNormalizer;

/// Ingest USDA PLANTS CSV file (species characteristics)
pub async fn ingest_usda_csv(
    db: &BotanicalDatabase,
    csv_path: &str,
) -> Result<(), DatabaseError> {
    let content = if csv_path.starts_with("http://") || csv_path.starts_with("https://") {
        reqwest::Client::new()
            .get(csv_path)
            .send()
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to download USDA CSV: {}", e)))?
            .bytes()
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to read response: {}", e)))?
            .to_vec()
    } else {
        tokio::fs::read(csv_path)
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to read CSV file: {}", e)))?
    };

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let csv_hash = hex::encode(hasher.finalize());

    let csv_text = String::from_utf8(content)
        .map_err(|e| DatabaseError::validation(format!("Invalid UTF-8 in CSV: {}", e)))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    let mut records: Vec<UsdsPlantRecord> = Vec::new();
    for result in reader.deserialize() {
        let record: UsdsPlantRecord = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping malformed row: {}", e);
                continue;
            }
        };
        records.push(record);
    }

    let csv_path = csv_path.to_string();
    let csv_hash_clone = csv_hash.clone();

    db.run_in_transaction(move |conn| {
        let mut row_count = 0u32;

        for record in records {
            let species_opt: Result<Option<String>, _> = conn.query_row(
                "SELECT id FROM species WHERE id = ? LIMIT 1",
                [&record.symbol as &dyn duckdb::ToSql],
                |row| row.get(0),
            );

            let species_id = match species_opt {
                Ok(Some(id)) => id,
                _ => continue,
            };

            if let Some(habit) = &record.growth_habit {
                let normalized = TraitNormalizer::normalize_growth_habit(habit);
                if !normalized.is_empty() {
                    let id = Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                         VALUES (?, ?, 'growth_habit', ?, NULL, NULL, NULL, 'USDA', 1)
                         ON CONFLICT (id) DO NOTHING",
                        [&id as &dyn duckdb::ToSql, &species_id, &normalized],
                    );
                }
            }

            if let Some(duration) = &record.duration {
                let normalized = TraitNormalizer::normalize_duration(duration);
                if !normalized.is_empty() {
                    let id = Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                         VALUES (?, ?, 'duration', ?, NULL, NULL, NULL, 'USDA', 1)
                         ON CONFLICT (id) DO NOTHING",
                        [&id as &dyn duckdb::ToSql, &species_id, &normalized],
                    );
                }
            }

            if let Some(height_str) = &record.mature_height_m {
                if let Some(height_val) = TraitNormalizer::parse_height_meters(height_str) {
                    let id = Uuid::new_v4().to_string();
                    let _ = conn.execute(
                        "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                         VALUES (?, ?, 'mature_height', NULL, ?, 'm', NULL, 'USDA', 1)
                         ON CONFLICT (id) DO NOTHING",
                        [&id as &dyn duckdb::ToSql, &species_id, &height_val.to_string()],
                    );
                }
            }

            for (req_type, value) in [
                ("drought_tolerance", &record.drought_tolerance),
                ("shade_tolerance", &record.shade_tolerance),
                ("salinity_tolerance", &record.salinity_tolerance),
                ("wetland_indicator", &record.wetland_indicator),
            ] {
                if let Some(val) = value {
                    let normalized = TraitNormalizer::normalize_tolerance(val);
                    if req_type == "wetland_indicator" {
                        let normalized_w = TraitNormalizer::normalize_wetland_indicator(val);
                        if !normalized_w.is_empty() {
                            let id = Uuid::new_v4().to_string();
                            let _ = conn.execute(
                                "INSERT INTO cultivation_requirements (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                                 VALUES (?, ?, ?, ?, NULL, NULL, NULL, 'USDA', 1)
                                 ON CONFLICT (id) DO NOTHING",
                                [&id as &dyn duckdb::ToSql, &species_id, &req_type, &normalized_w],
                            );
                        }
                    } else if !normalized.is_empty() {
                        let id = Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT INTO cultivation_requirements (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                             VALUES (?, ?, ?, ?, NULL, NULL, NULL, 'USDA', 1)
                             ON CONFLICT (id) DO NOTHING",
                            [&id as &dyn duckdb::ToSql, &species_id, &req_type, &normalized],
                        );
                    }
                }
            }

            row_count += 1;
        }

        let prov_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
             VALUES (?, NULL, 'USDA_PLANTS_CSV', ?, 'Public Domain', current_timestamp, ?)",
            [&prov_id as &dyn duckdb::ToSql, &csv_path, &csv_hash_clone],
        ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

        println!("USDA CSV import complete: {} rows processed, CSV hash: {}", row_count, csv_hash_clone);
        Ok(())
    }).await
}

#[derive(Debug, serde::Deserialize)]
struct UsdsPlantRecord {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    growth_habit: Option<String>,
    #[serde(default)]
    duration: Option<String>,
    #[serde(default)]
    mature_height_m: Option<String>,
    #[serde(default)]
    drought_tolerance: Option<String>,
    #[serde(default)]
    shade_tolerance: Option<String>,
    #[serde(default)]
    salinity_tolerance: Option<String>,
    #[serde(default)]
    wetland_indicator: Option<String>,
}
