/// USDA bulk CSV ingestion from downloaded PLANTS datasets
use crate::error::DatabaseError;
use sqlx::SqlitePool;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use super::normalization::TraitNormalizer;

/// Ingest USDA PLANTS CSV file (species characteristics).
/// CSV expected columns: symbol, growth_habit, duration, mature_height_m, drought_tolerance, shade_tolerance, salinity_tolerance, wetland_indicator
/// 
/// Note: CSV file path can be local or remote via USDA_CSV_PATH env var.
pub async fn ingest_usda_csv(
    pool: &SqlitePool,
    csv_path: &str,
) -> Result<(), DatabaseError> {
    let content = if csv_path.starts_with("http://") || csv_path.starts_with("https://") {
        // Download from URL
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
        // Load from file
        tokio::fs::read(csv_path)
            .await
            .map_err(|e| DatabaseError::validation(format!("Failed to read CSV file: {}", e)))?
    };

    // Compute hash of entire CSV for audit trail
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let csv_hash = hex::encode(hasher.finalize());

    let csv_text = String::from_utf8(content)
        .map_err(|e| DatabaseError::validation(format!("Invalid UTF-8 in CSV: {}", e)))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_text.as_bytes());

    let mut tx = pool.begin().await?;
    let mut row_count = 0;
    let mut error_count = 0;

    for result in reader.deserialize() {
        let record: UsdsPlantRecord = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping malformed row: {}", e);
                error_count += 1;
                continue;
            }
        };

        // Find species by USDA symbol (assumed to exist; skip if not)
        let species_opt: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM species WHERE usda_symbol = ?1 LIMIT 1"
        )
        .bind(&record.symbol)
        .fetch_optional(&mut *tx)
        .await?;

        let Some((species_id,)) = species_opt else {
            // Skip species not in DB; could batch-load them later
            continue;
        };

        // Traits
        if let Some(habit) = &record.growth_habit {
            let normalized = TraitNormalizer::normalize_growth_habit(habit);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO traits
                       (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                       VALUES (?1, ?2, 'growth_habit', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        if let Some(duration) = &record.duration {
            let normalized = TraitNormalizer::normalize_duration(duration);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO traits
                       (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                       VALUES (?1, ?2, 'duration', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        if let Some(height_str) = &record.mature_height_m {
            if let Some(height_val) = TraitNormalizer::parse_height_meters(height_str) {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO traits
                       (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                       VALUES (?1, ?2, 'mature_height', NULL, ?3, 'm', NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(height_val)
                .execute(&mut *tx)
                .await;
            }
        }

        // Tolerances
        if let Some(drought) = &record.drought_tolerance {
            let normalized = TraitNormalizer::normalize_tolerance(drought);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO cultivation_requirements
                       (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                       VALUES (?1, ?2, 'drought_tolerance', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        if let Some(shade) = &record.shade_tolerance {
            let normalized = TraitNormalizer::normalize_tolerance(shade);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO cultivation_requirements
                       (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                       VALUES (?1, ?2, 'shade_tolerance', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        if let Some(salinity) = &record.salinity_tolerance {
            let normalized = TraitNormalizer::normalize_tolerance(salinity);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO cultivation_requirements
                       (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                       VALUES (?1, ?2, 'salinity_tolerance', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        if let Some(wetland) = &record.wetland_indicator {
            let normalized = TraitNormalizer::normalize_wetland_indicator(wetland);
            if !normalized.is_empty() {
                let id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    r#"INSERT OR IGNORE INTO cultivation_requirements
                       (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                       VALUES (?1, ?2, 'wetland_indicator', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
                )
                .bind(&id)
                .bind(&species_id)
                .bind(&normalized)
                .execute(&mut *tx)
                .await;
            }
        }

        row_count += 1;
    }

    // Record CSV import in source log
    let prov_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO provenance
           (id, species_id, source, source_record_id, license, retrieved_at, hash)
           VALUES (?1, NULL, 'USDA_PLANTS_CSV', ?2, 'Public Domain', datetime('now'), ?3)"#,
    )
    .bind(&prov_id)
    .bind("bulk_import")
    .bind(&csv_hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    println!(
        "USDA CSV import complete: {} rows processed, {} errors, CSV hash: {}",
        row_count, error_count, csv_hash
    );
    Ok(())
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
