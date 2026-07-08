use uuid::Uuid;
use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;
use sha2::{Sha256, Digest};

#[derive(Debug, serde::Deserialize)]
pub struct GbifVernacularName {
    #[serde(rename = "vernacularName")]
    pub vernacular_name: Option<String>,
    pub language: Option<String>,
    #[serde(rename = "isPreferredName")]
    pub is_preferred_name: Option<bool>,
    pub source: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GbifNameResponse {
    #[serde(default)]
    pub results: Vec<GbifVernacularName>,
}

pub struct GbifClient {
    pub http: reqwest::Client,
    pub base_url: String,
}

impl Default for GbifClient {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: "https://api.gbif.org/v1".to_string(),
        }
    }
}

impl GbifClient {
    pub async fn fetch_vernacular_names(&self, gbif_id: &str) -> anyhow::Result<(Vec<GbifVernacularName>, String)> {
        let url = format!("{}/species/{}/vernacularNames", self.base_url, gbif_id);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("GBIF vernacular request failed: {} {}", status, url);
        }
        let bytes = resp.bytes().await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());
        let parsed = serde_json::from_slice::<GbifNameResponse>(&bytes)?;
        Ok((parsed.results, hash))
    }
}

/// Ingest GBIF vernacular names for an existing species
pub async fn ingest_gbif_vernacular(
    db: &BotanicalDatabase,
    species_id: &str,
    gbif_id: &str,
    client: &GbifClient,
) -> Result<(), DatabaseError> {
    let (names, payload_hash) = client
        .fetch_vernacular_names(gbif_id)
        .await
        .map_err(|e| DatabaseError::validation(format!("GBIF fetch failed: {}", e)))?;

    let species_id = species_id.to_string();
    let gbif_id = gbif_id.to_string();

    db.run_in_transaction(move |conn| {
        let mut seen_languages: std::collections::HashSet<String> = std::collections::HashSet::new();

        for n in names.into_iter() {
            let name = n.vernacular_name.unwrap_or_default();
            if name.is_empty() { continue; }

            let lang = n.language.unwrap_or_else(|| "unknown".to_string());
            let is_preferred = n.is_preferred_name.unwrap_or(false);

            if seen_languages.contains(&lang) { continue; }
            seen_languages.insert(lang.clone());

            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO vernacular_names (id, species_id, name, language_code, is_primary, source, created_at)
                 VALUES (?, ?, ?, ?, ?, 'GBIF', current_timestamp)
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &name, &lang, &(is_preferred as i32).to_string()],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        let prov_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
             VALUES (?, ?, 'GBIF', ?, 'CC BY 4.0', current_timestamp, ?)",
            [&prov_id as &dyn duckdb::ToSql, &species_id, &gbif_id, &payload_hash],
        ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

        Ok(())
    }).await
}
