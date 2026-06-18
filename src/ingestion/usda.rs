use sqlx::SqlitePool;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use crate::error::DatabaseError;

/// Minimal USDA client. Base URL is configurable to adapt to actual USDA endpoints.
pub struct UsdaClient {
    pub http: reqwest::Client,
    pub base_url: String,
}

impl Default for UsdaClient {
    fn default() -> Self {
        // Placeholder default. Replace base_url with the authoritative USDA PLANTS JSON endpoint.
        Self { http: reqwest::Client::new(), base_url: "https://plantsdb.xyz/api".to_string() }
    }
}

/// Fetch a USDA record for a taxon symbol or id.
pub async fn fetch_usda_payload(client: &UsdaClient, taxon_key: &str) -> anyhow::Result<(serde_json::Value, String)> {
    let url = format!("{}/v1/plants/{}", client.base_url, taxon_key);
    let resp = client.http.get(&url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("USDA request failed: {} {}", status, url);
    }
    let bytes = resp.bytes().await?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    Ok((value, hash))
}

/// Ingest USDA traits and tolerances into traits and cultivation_requirements tables.
/// - species_id must already exist in `species`
/// - `taxon_key` is a USDA symbol or identifier that resolves at the configured endpoint
pub async fn ingest_usda_traits(
    pool: &SqlitePool,
    species_id: &str,
    taxon_key: &str,
    client: &UsdaClient,
) -> Result<(), DatabaseError> {
    let (payload, hash) = fetch_usda_payload(client, taxon_key)
        .await
        .map_err(|e| DatabaseError::validation(format!("USDA fetch failed: {}", e)))?;

    let mut tx = pool.begin().await?;

    // Helper to extract string field by common USDA names
    let get_str = |key: &str| -> Option<String> {
        payload.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    let get_num = |key: &str| -> Option<f64> {
        payload.get(key).and_then(|v| v.as_f64())
    };

    // Traits (text/numeric)
    if let Some(habit) = get_str("growth_habit") {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO traits
               (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
               VALUES (?1, ?2, 'growth_habit', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
        )
        .bind(&id).bind(species_id).bind(habit)
        .execute(&mut *tx).await?;
    }

    if let Some(duration) = get_str("duration") {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO traits
               (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
               VALUES (?1, ?2, 'duration', ?3, NULL, NULL, NULL, 'USDA', 1)"#,
        )
        .bind(&id).bind(species_id).bind(duration)
        .execute(&mut *tx).await?;
    }

    if let Some(height_m) = get_num("mature_height_m") {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO traits
               (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
               VALUES (?1, ?2, 'mature_height', NULL, ?3, 'm', NULL, 'USDA', 1)"#,
        )
        .bind(&id).bind(species_id).bind(height_m)
        .execute(&mut *tx).await?;
    }

    // Tolerances as cultivation requirements
    for (req_type, key) in [
        ("drought_tolerance", "drought_tolerance"),
        ("shade_tolerance", "shade_tolerance"),
        ("salinity_tolerance", "salinity_tolerance"),
        ("wetland_indicator", "wetland_indicator"),
    ] {
        if let Some(val) = get_str(key) {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"INSERT OR IGNORE INTO cultivation_requirements
                   (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                   VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, 'USDA', 1)"#,
            )
            .bind(&id).bind(species_id).bind(req_type).bind(val)
            .execute(&mut *tx).await?;
        }
    }

    // Provenance
    let prov_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO provenance
           (id, species_id, source, source_record_id, license, retrieved_at, hash)
           VALUES (?1, ?2, 'USDA', ?3, 'Public Domain', datetime('now'), ?4)"#,
    )
    .bind(&prov_id)
    .bind(species_id)
    .bind(taxon_key)
    .bind(&hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
