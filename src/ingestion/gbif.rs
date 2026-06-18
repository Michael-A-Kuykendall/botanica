use sqlx::SqlitePool;
use uuid::Uuid;
use crate::error::DatabaseError;
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
/// Dedup logic: keep first preferred name per language; silently skip later duplicates
pub async fn ingest_gbif_vernacular(
    pool: &SqlitePool,
    species_id: &str,
    gbif_id: &str,
    client: &GbifClient,
) -> Result<(), DatabaseError> {
    let (names, payload_hash) = client
        .fetch_vernacular_names(gbif_id)
        .await
        .map_err(|e| DatabaseError::validation(format!("GBIF fetch failed: {}", e)))?;

    let mut tx = pool.begin().await?;
    
    // Dedup: track languages seen to keep first preferred per language
    let mut seen_languages: std::collections::HashSet<String> = std::collections::HashSet::new();

    for n in names.into_iter() {
        let name = n.vernacular_name.unwrap_or_default();
        if name.is_empty() { continue; }
        
        let lang = n.language.unwrap_or_else(|| "unknown".to_string());
        let is_preferred = n.is_preferred_name.unwrap_or(false);
        
        // Skip if we've already seen a name in this language
        if seen_languages.contains(&lang) {
            continue;
        }
        
        // Mark language as seen (on first encounter, preferred or not)
        seen_languages.insert(lang.clone());
        
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO vernacular_names
               (id, species_id, name, language_code, is_primary, source, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, 'GBIF', datetime('now'))"#,
        )
        .bind(&id)
        .bind(species_id)
        .bind(&name)
        .bind(&lang)
        .bind(is_preferred as i32)
        .execute(&mut *tx)
        .await?;
    }

    // Provenance for GBIF pull
    let prov_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO provenance
           (id, species_id, source, source_record_id, license, retrieved_at, hash)
           VALUES (?1, ?2, 'GBIF', ?3, 'CC BY 4.0', datetime('now'), NULL)"#,
    )
    .bind(&prov_id)
    .bind(species_id)
    .bind(gbif_id)
    .execute(&mut *tx)
    .await?;
    // Update hash
    sqlx::query("UPDATE provenance SET hash = ?1 WHERE id = ?2")
        .bind(&payload_hash)
        .bind(&prov_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}
