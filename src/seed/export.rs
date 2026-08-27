//! Export silver tables to parquet.

use crate::database::BotanicalDatabase;
use crate::error::DatabaseError;
use std::path::Path;

/// Tables exported as silver parquet (L1+L2 only; no L3 inventory rows expected).
pub const SILVER_TABLES: &[&str] = &[
    "families",
    "genera",
    "species",
    "species_identifiers",
    "cultivars",
    "synonyms",
    "vernacular_names",
    "distribution_regions",
    "traits",
    "seasonal_characteristics",
    "cultivation_requirements",
    "ecological_interactions",
    "uses",
    "media",
    "provenance",
    "ingest_quarantine",
];

/// COPY each silver table to `{out_dir}/{table}.parquet`.
pub async fn export_silver_parquet(
    db: &BotanicalDatabase,
    out_dir: &Path,
) -> Result<Vec<String>, DatabaseError> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| DatabaseError::validation(format!("create silver dir: {}", e)))?;

    let conn = db.conn().await;
    let mut written = Vec::new();

    for table in SILVER_TABLES {
        let path = out_dir.join(format!("{}.parquet", table));
        // Use forward slashes for DuckDB on Windows
        let path_str = path.to_string_lossy().replace('\\', "/");
        let sql = format!(
            "COPY (SELECT * FROM {}) TO '{}' (FORMAT PARQUET)",
            table, path_str
        );
        match conn.execute(&sql, []) {
            Ok(_) => written.push(path_str),
            Err(e) => {
                // empty / missing table: skip with note
                log::warn!("parquet export skip {}: {}", table, e);
            }
        }
    }

    Ok(written)
}

/// Rebuild DuckDB tables from silver parquet directory (loader path).
pub async fn load_silver_parquet(
    db: &BotanicalDatabase,
    silver_dir: &Path,
) -> Result<usize, DatabaseError> {
    db.migrate().await?;
    let conn = db.conn().await;
    let mut loaded = 0usize;

    for table in SILVER_TABLES {
        let path = silver_dir.join(format!("{}.parquet", table));
        if !path.exists() {
            continue;
        }
        let path_str = path.to_string_lossy().replace('\\', "/");
        // Clear and reload for idempotent load
        let _ = conn.execute(&format!("DELETE FROM {}", table), []);
        let sql = format!(
            "INSERT INTO {} SELECT * FROM read_parquet('{}')",
            table, path_str
        );
        conn.execute(&sql, [])
            .map_err(|e| DatabaseError::DuckDbError(format!("load {}: {}", table, e)))?;
        loaded += 1;
    }
    Ok(loaded)
}
