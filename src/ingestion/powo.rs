use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;
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
    pub async fn fetch_species_detail(&self, powo_id: &str) -> anyhow::Result<(PowoSpeciesDetail, String)> {
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
pub async fn ingest_powo_for_species(
    db: &BotanicalDatabase,
    species_id: &str,
    powo_id: &str,
    client: &PowoClient,
) -> Result<(), DatabaseError> {
    let (detail, payload_hash) = client
        .fetch_species_detail(powo_id)
        .await
        .map_err(|e| DatabaseError::validation(format!("POWO fetch failed: {}", e)))?;

    let species_id = species_id.to_string();
    let powo_id = powo_id.to_string();

    db.run_in_transaction(move |conn| {
        // Synonyms
        for syn in detail.synonyms.into_iter() {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO synonyms (id, species_id, synonym_name, authorship, source, source_record_id)
                 VALUES (?, ?, ?, ?, 'POWO', ?)
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &syn.name.unwrap_or_default(), &syn.authorship.unwrap_or_default(), &syn.id.unwrap_or_default()],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        // Distribution
        for d in detail.distribution.into_iter() {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO distribution_regions (id, species_id, region_code, region_source, notes, source)
                 VALUES (?, ?, ?, ?, NULL, 'POWO')
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &d.region_code.unwrap_or_default(), &d.source.unwrap_or_else(|| "WGSRPD".to_string())],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        // Uses
        for u in detail.uses.into_iter() {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO uses (id, species_id, use_category, description, source)
                 VALUES (?, ?, ?, ?, 'POWO')
                 ON CONFLICT (id) DO NOTHING",
                [&id as &dyn duckdb::ToSql, &species_id, &u.category.unwrap_or_else(|| "unspecified".to_string()), &u.description.unwrap_or_default()],
            ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
        }

        // Provenance
        let prov_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
             VALUES (?, ?, 'POWO', ?, 'CC BY 4.0', current_timestamp, ?)",
            [&prov_id as &dyn duckdb::ToSql, &species_id, &powo_id, &payload_hash],
        ).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;

        Ok(())
    }).await
}
