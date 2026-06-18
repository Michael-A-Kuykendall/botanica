use crate::error::DatabaseError;
use sqlx::SqlitePool;
use uuid::Uuid;
use sha2::{Sha256, Digest};

#[derive(Clone, Debug)]
pub struct PowoClient {
    pub base_url: String,
    pub http: reqwest::Client,
}

impl Default for PowoClient {
    fn default() -> Self {
        Self {
            base_url: "https://powo.science.kew.org/api/2".to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct PowoSynonym {
    pub name: Option<String>,
    pub authorship: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PowoUseEntry {
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PowoDistributionEntry {
    pub region_code: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PowoSpeciesDetail {
    pub name: Option<String>,
    pub authorship: Option<String>,
    #[serde(default)]
    pub synonyms: Vec<PowoSynonym>,
    #[serde(default)]
    pub distribution: Vec<PowoDistributionEntry>,
    #[serde(default)]
    pub uses: Vec<PowoUseEntry>,
}

impl PowoClient {
    /// Fetch species details (accepted name, synonyms, distribution, uses)
    pub async fn fetch_species_detail(&self, powo_id: &str) -> anyhow::Result<(PowoSpeciesDetail, String)> {
        // NOTE: POWO API paths vary; keep configurable and resilient.
        // Example path pattern (to adjust as needed): /taxon/{id}
        let url = format!("{}/taxon/{}", self.base_url, powo_id);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("POWO request failed: {} {}", status, url);
        }
        let bytes = resp.bytes().await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hex::encode(hasher.finalize());
        let detail = serde_json::from_slice::<PowoSpeciesDetail>(&bytes)?;
        Ok((detail, hash))
    }
}

/// Ingest POWO data for an existing species_id
/// - Assumes the species row already exists in `species` table
/// - Inserts into synonyms, distribution_regions, uses, provenance
pub async fn ingest_powo_for_species(
    pool: &SqlitePool,
    species_id: &str,
    powo_id: &str,
    client: &PowoClient,
) -> Result<(), DatabaseError> {
    let (detail, payload_hash) = client
        .fetch_species_detail(powo_id)
        .await
        .map_err(|e| DatabaseError::validation(format!("POWO fetch failed: {}", e)))?;

    let mut tx = pool.begin().await?;

    // Synonyms
    for syn in detail.synonyms.into_iter() {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO synonyms
               (id, species_id, synonym_name, authorship, source, source_record_id)
               VALUES (?1, ?2, ?3, ?4, 'POWO', ?5)"#,
        )
        .bind(&id)
        .bind(species_id)
        .bind(syn.name.unwrap_or_default())
        .bind(syn.authorship.unwrap_or_default())
        .bind(syn.id.unwrap_or_default())
        .execute(&mut *tx)
        .await?;
    }

    // Distribution
    for d in detail.distribution.into_iter() {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO distribution_regions
               (id, species_id, region_code, region_source, notes, source)
               VALUES (?1, ?2, ?3, ?4, NULL, 'POWO')"#,
        )
        .bind(&id)
        .bind(species_id)
        .bind(d.region_code.unwrap_or_default())
        .bind(d.source.unwrap_or_else(|| "WGSRPD".to_string()))
        .execute(&mut *tx)
        .await?;
    }

    // Uses
    for u in detail.uses.into_iter() {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT OR IGNORE INTO uses
               (id, species_id, use_category, description, source)
               VALUES (?1, ?2, ?3, ?4, 'POWO')"#,
        )
        .bind(&id)
        .bind(species_id)
        .bind(u.category.unwrap_or_else(|| "unspecified".to_string()))
        .bind(u.description.unwrap_or_default())
        .execute(&mut *tx)
        .await?;
    }

    // Provenance (record the fetch)
    let prov_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO provenance
           (id, species_id, source, source_record_id, license, retrieved_at, hash)
           VALUES (?1, ?2, 'POWO', ?3, 'CC BY 4.0', datetime('now'), NULL)"#,
    )
    .bind(&prov_id)
    .bind(species_id)
    .bind(powo_id)
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
