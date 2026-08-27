use crate::error::DatabaseError;
use crate::database::BotanicalDatabase;
use super::schemas::{
    SCHEMA_VERSION, SCHEMA_META_TABLE_SQL, REQUIRED_TABLES,
    FAMILIES_TABLE_SQL, GENERA_TABLE_SQL, SPECIES_TABLE_SQL,
    SPECIES_IDENTIFIERS_TABLE_SQL, CULTIVARS_TABLE_SQL, INGEST_QUARANTINE_TABLE_SQL,
    PLANTS_TABLE_SQL, PLANT_PHOTOS_TABLE_SQL, CARE_ACTIVITIES_TABLE_SQL,
    ENVIRONMENTS_TABLE_SQL, CULTIVATION_RECORDS_TABLE_SQL,
    SYNONYMS_TABLE_SQL, VERNACULAR_NAMES_TABLE_SQL, DISTRIBUTION_REGIONS_TABLE_SQL,
    TRAITS_TABLE_SQL, SEASONAL_CHARACTERISTICS_TABLE_SQL, CULTIVATION_REQUIREMENTS_TABLE_SQL,
    ECOLOGICAL_INTERACTIONS_TABLE_SQL, USES_TABLE_SQL, MEDIA_TABLE_SQL, PROVENANCE_TABLE_SQL,
};

fn exec(conn: &duckdb::Connection, sql: &str) -> Result<(), DatabaseError> {
    conn.execute(sql, [])
        .map_err(|e| DatabaseError::migration(format!("{} — sql: {}", e, sql.chars().take(120).collect::<String>())))?;
    Ok(())
}

/// Run all database migrations
pub async fn run_migrations(db: &BotanicalDatabase) -> Result<(), DatabaseError> {
    let conn = db.conn().await;

    exec(&conn, SCHEMA_META_TABLE_SQL)?;

    // L1 taxonomy
    exec(&conn, FAMILIES_TABLE_SQL)?;
    exec(&conn, GENERA_TABLE_SQL)?;
    exec(&conn, SPECIES_TABLE_SQL)?;
    exec(&conn, SPECIES_IDENTIFIERS_TABLE_SQL)?;
    exec(&conn, CULTIVARS_TABLE_SQL)?;
    exec(&conn, INGEST_QUARANTINE_TABLE_SQL)?;

    // L2 horticultural reference
    exec(&conn, SYNONYMS_TABLE_SQL)?;
    exec(&conn, VERNACULAR_NAMES_TABLE_SQL)?;
    exec(&conn, DISTRIBUTION_REGIONS_TABLE_SQL)?;
    exec(&conn, TRAITS_TABLE_SQL)?;
    exec(&conn, SEASONAL_CHARACTERISTICS_TABLE_SQL)?;
    exec(&conn, CULTIVATION_REQUIREMENTS_TABLE_SQL)?;
    exec(&conn, ECOLOGICAL_INTERACTIONS_TABLE_SQL)?;
    exec(&conn, USES_TABLE_SQL)?;
    exec(&conn, MEDIA_TABLE_SQL)?;
    exec(&conn, PROVENANCE_TABLE_SQL)?;

    // L3 inventory (empty in OSS seed) — cultivars must exist first for FK
    exec(&conn, PLANTS_TABLE_SQL)?;
    exec(&conn, PLANT_PHOTOS_TABLE_SQL)?;
    exec(&conn, CARE_ACTIVITIES_TABLE_SQL)?;
    exec(&conn, ENVIRONMENTS_TABLE_SQL)?;
    exec(&conn, CULTIVATION_RECORDS_TABLE_SQL)?;

    // Best-effort column adds for DBs created before 0.4.0.
    // Each statement is isolated: failure must not poison later work (DuckDB aborts txn on error).
    let alters = [
        "ALTER TABLE species ADD COLUMN IF NOT EXISTS scientific_name VARCHAR",
        "ALTER TABLE species ADD COLUMN IF NOT EXISTS taxonomic_status VARCHAR DEFAULT 'accepted'",
        "ALTER TABLE species ADD COLUMN IF NOT EXISTS rank VARCHAR DEFAULT 'species'",
        "ALTER TABLE plants ADD COLUMN IF NOT EXISTS cultivar_id VARCHAR",
        "ALTER TABLE plants ADD COLUMN IF NOT EXISTS health_status VARCHAR DEFAULT 'unknown'",
        "ALTER TABLE plants ADD COLUMN IF NOT EXISTS user_id VARCHAR",
        "ALTER TABLE plants ADD COLUMN IF NOT EXISTS device_id VARCHAR",
        "ALTER TABLE plants ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP",
    ];
    for sql in &alters {
        if let Err(e) = conn.execute(sql, []) {
            // Rollback aborted state if any, then continue
            let _ = conn.execute("ROLLBACK", []);
            log::debug!("migration alter skipped: {} ({})", sql, e);
        }
    }

    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_genera_family ON genera(family_id)",
        "CREATE INDEX IF NOT EXISTS idx_species_genus ON species(genus_id)",
        "CREATE INDEX IF NOT EXISTS idx_species_scientific ON species(scientific_name)",
        "CREATE INDEX IF NOT EXISTS idx_species_status ON species(taxonomic_status)",
        "CREATE INDEX IF NOT EXISTS idx_sid_species ON species_identifiers(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_sid_source ON species_identifiers(source)",
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_sid_source_ext ON species_identifiers(source, external_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultivars_species ON cultivars(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_quarantine_source ON ingest_quarantine(source)",
        "CREATE INDEX IF NOT EXISTS idx_synonyms_species ON synonyms(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_vernacular_species ON vernacular_names(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_vernacular_lang ON vernacular_names(language_code)",
        "CREATE INDEX IF NOT EXISTS idx_distribution_species ON distribution_regions(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_distribution_region ON distribution_regions(region_code)",
        "CREATE INDEX IF NOT EXISTS idx_traits_species ON traits(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_traits_name ON traits(trait_name)",
        "CREATE INDEX IF NOT EXISTS idx_seasonal_species ON seasonal_characteristics(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultreq_species ON cultivation_requirements(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultreq_type ON cultivation_requirements(requirement_type)",
        "CREATE INDEX IF NOT EXISTS idx_uses_species ON uses(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_media_species ON media(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_provenance_species ON provenance(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_provenance_source ON provenance(source)",
        "CREATE INDEX IF NOT EXISTS idx_plants_species ON plants(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_plants_cultivar ON plants(cultivar_id)",
        "CREATE INDEX IF NOT EXISTS idx_plants_health ON plants(health_status)",
        "CREATE INDEX IF NOT EXISTS idx_plants_user ON plants(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_care_plant ON care_activities(plant_id)",
        "CREATE INDEX IF NOT EXISTS idx_env_plant ON environments(plant_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultrec_plant ON cultivation_records(plant_id)",
    ];

    for idx_sql in &indexes {
        exec(&conn, idx_sql)?;
    }

    // Stamp schema version (portable upsert)
    let _ = conn.execute("DELETE FROM schema_meta WHERE key = 'schema_version'", []);
    conn.execute(
        "INSERT INTO schema_meta (key, value) VALUES ('schema_version', ?)",
        [SCHEMA_VERSION],
    )
    .map_err(|e| DatabaseError::migration(e.to_string()))?;

    Ok(())
}

/// Validate that all required tables exist
pub async fn validate_migrations(db: &BotanicalDatabase) -> Result<bool, DatabaseError> {
    let conn = db.conn().await;
    for table in REQUIRED_TABLES {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [table],
                |row| row.get(0),
            )
            .map_err(|e| DatabaseError::migration(e.to_string()))?;
        if count == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// List required tables and whether each exists
pub async fn get_migration_status(db: &BotanicalDatabase) -> Result<Vec<String>, DatabaseError> {
    let conn = db.conn().await;
    let mut status = Vec::new();
    for table in REQUIRED_TABLES {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [table],
                |row| row.get(0),
            )
            .map_err(|e| DatabaseError::migration(e.to_string()))?;
        status.push(format!(
            "{}: {}",
            table,
            if count > 0 { "ok" } else { "missing" }
        ));
    }
    Ok(status)
}

/// Read schema version from schema_meta (or report unknown)
pub async fn check_schema_version(db: &BotanicalDatabase) -> Result<String, DatabaseError> {
    let conn = db.conn().await;
    match conn.query_row(
        "SELECT value FROM schema_meta WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(v) => Ok(v),
        Err(duckdb::Error::QueryReturnedNoRows) => Ok("unknown".to_string()),
        Err(e) => Err(DatabaseError::migration(e.to_string())),
    }
}
