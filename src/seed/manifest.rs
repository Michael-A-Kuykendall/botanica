use crate::database::BotanicalDatabase;
use crate::error::DatabaseError;
use crate::migrations::schemas::SCHEMA_VERSION;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SeedManifest {
    pub artifact: String,
    pub built_at: String,
    pub engine: String,
    pub schema_version: String,
    pub counts: SeedCounts,
    pub sources: Vec<SeedSource>,
    pub scope: String,
    pub l3_rows: i64,
    pub silver_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SeedCounts {
    pub families: i64,
    pub genera: i64,
    pub species: i64,
    pub cultivars: i64,
    pub traits: i64,
    pub vernacular_names: i64,
    pub cultivation_requirements: i64,
    pub provenance: i64,
    pub quarantine: i64,
    pub plants: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeedSource {
    pub name: String,
    pub license: String,
    pub record_count: i64,
    pub notes: String,
}

fn count(conn: &duckdb::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
        .unwrap_or(0)
}

/// Collect counts + write MANIFEST json.
pub async fn write_manifest(
    db: &BotanicalDatabase,
    path: &Path,
    artifact: &str,
    silver_files: Vec<String>,
    sources: Vec<SeedSource>,
) -> Result<SeedManifest, DatabaseError> {
    let conn = db.conn().await;
    let counts = SeedCounts {
        families: count(&conn, "families"),
        genera: count(&conn, "genera"),
        species: count(&conn, "species"),
        cultivars: count(&conn, "cultivars"),
        traits: count(&conn, "traits"),
        vernacular_names: count(&conn, "vernacular_names"),
        cultivation_requirements: count(&conn, "cultivation_requirements"),
        provenance: count(&conn, "provenance"),
        quarantine: count(&conn, "ingest_quarantine"),
        plants: count(&conn, "plants"),
    };

    let manifest = SeedManifest {
        artifact: artifact.to_string(),
        built_at: chrono::Utc::now().to_rfc3339(),
        engine: "duckdb".to_string(),
        schema_version: SCHEMA_VERSION.to_string(),
        counts,
        sources,
        scope: "cultivated_human_use_gate2_pilot".to_string(),
        l3_rows: count(&conn, "plants"),
        silver_files,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DatabaseError::validation(format!("manifest dir: {}", e)))?;
    }
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| DatabaseError::validation(format!("manifest json: {}", e)))?;
    std::fs::write(path, json)
        .map_err(|e| DatabaseError::validation(format!("write manifest: {}", e)))?;

    Ok(manifest)
}
