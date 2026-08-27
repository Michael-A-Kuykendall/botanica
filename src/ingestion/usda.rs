use uuid::Uuid;
use sha2::{Sha256, Digest};
use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;

pub struct UsdaClient {
    pub http: reqwest::Client,
    pub base_url: String,
}

impl Default for UsdaClient {
    fn default() -> Self {
        Self { http: reqwest::Client::new(), base_url: "https://plantsdb.xyz/api".to_string() }
    }
}

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

/// Ingest USDA traits and tolerances
pub async fn ingest_usda_traits(
    db: &BotanicalDatabase,
    species_id: &str,
    taxon_key: &str,
    client: &UsdaClient,
) -> Result<(), DatabaseError> {
    let (payload, hash) = fetch_usda_payload(client, taxon_key)
        .await
        .map_err(|e| DatabaseError::validation(format!("USDA fetch failed: {}", e)))?;

    let species_id = species_id.to_string();
    let taxon_key = taxon_key.to_string();

    db.run_in_transaction(move |conn| {
        let get_str = |key: &str| -> Option<String> {
            payload.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
        };
        let get_num = |key: &str| -> Option<f64> {
            payload.get(key).and_then(|v| v.as_f64())
        };

        if let Some(habit) = get_str("growth_habit") {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                 VALUES (?, ?, 'growth_habit', ?, NULL, NULL, NULL, 'USDA', 1)
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &habit],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        if let Some(duration) = get_str("duration") {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                 VALUES (?, ?, 'duration', ?, NULL, NULL, NULL, 'USDA', 1)
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &duration],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        if let Some(height_m) = get_num("mature_height_m") {
            let id = Uuid::new_v4().to_string();
            let height_str = height_m.to_string();
            conn.execute(
                "INSERT INTO traits (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability)
                 VALUES (?, ?, 'mature_height', NULL, ?, 'm', NULL, 'USDA', 1)
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &height_str],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        for (req_type, key) in [
            ("drought_tolerance", "drought_tolerance"),
            ("shade_tolerance", "shade_tolerance"),
            ("salinity_tolerance", "salinity_tolerance"),
            ("wetland_indicator", "wetland_indicator"),
        ] {
            if let Some(val) = get_str(key) {
                let id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO cultivation_requirements (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability)
                     VALUES (?, ?, ?, ?, NULL, NULL, NULL, 'USDA', 1)
                     ON CONFLICT (id) DO NOTHING",
                    [&id as &dyn duckdb::ToSql, &species_id, &req_type, &val],
                ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
            }
        }

        let prov_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
             VALUES (?, ?, 'USDA', ?, 'Public Domain', current_timestamp, ?)",
            [&prov_id as &dyn duckdb::ToSql, &species_id, &taxon_key, &hash],
        ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

        Ok(())
    }).await
}
