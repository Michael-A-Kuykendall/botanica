use sqlx::{SqlitePool, query};
use crate::error::DatabaseError;

pub mod runner;
pub mod schemas;


/// Initialize the database with all required tables
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DatabaseError> {
    // Create families table
    query(r#"
        CREATE TABLE IF NOT EXISTS families (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            authority TEXT
        )
    "#)
    .execute(pool)
    .await?;

    // Create genera table
    query(r#"
        CREATE TABLE IF NOT EXISTS genera (
            id TEXT PRIMARY KEY,
            family_id TEXT NOT NULL,
            name TEXT NOT NULL,
            authority TEXT,
            FOREIGN KEY (family_id) REFERENCES families(id)
        )
    "#)
    .execute(pool)
    .await?;

    // Create species table
    query(r#"
        CREATE TABLE IF NOT EXISTS species (
            id TEXT PRIMARY KEY,
            genus_id TEXT NOT NULL,
            specific_epithet TEXT NOT NULL,
            authority TEXT,
            publication_year INTEGER,
            conservation_status TEXT,
            FOREIGN KEY (genus_id) REFERENCES genera(id)
        )
    "#)
    .execute(pool)
    .await?;

    // Create specimens table
    query(r#"
        CREATE TABLE IF NOT EXISTS specimens (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            collector TEXT,
            collection_date TEXT,
            location TEXT,
            notes TEXT,
            FOREIGN KEY (species_id) REFERENCES species(id)
        )
    "#)
    .execute(pool)
    .await?;

    // --- Horticultural reference tables (Phase 1) ---
    // Synonyms of accepted species names
    query(r#"
        CREATE TABLE IF NOT EXISTS synonyms (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            synonym_name TEXT NOT NULL,
            authorship TEXT,
            source TEXT NOT NULL,
            source_record_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Vernacular (common) names
    query(r#"
        CREATE TABLE IF NOT EXISTS vernacular_names (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            name TEXT NOT NULL,
            language_code TEXT,
            is_primary INTEGER NOT NULL DEFAULT 0,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Geographic distribution regions (WGSRPD codes, USDA regions, etc.)
    query(r#"
        CREATE TABLE IF NOT EXISTS distribution_regions (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            region_code TEXT NOT NULL,
            region_source TEXT,
            notes TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Species traits (structured + numeric)
    query(r#"
        CREATE TABLE IF NOT EXISTS traits (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            trait_name TEXT NOT NULL,
            trait_value_text TEXT,
            trait_value_numeric REAL,
            units TEXT,
            method TEXT,
            source TEXT NOT NULL,
            reliability INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Seasonal characteristics (flowering, fruiting, dormancy)
    query(r#"
        CREATE TABLE IF NOT EXISTS seasonal_characteristics (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            season TEXT NOT NULL,
            characteristic_type TEXT NOT NULL,
            value TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Cultivation requirements (light, soil, pH, moisture)
    query(r#"
        CREATE TABLE IF NOT EXISTS cultivation_requirements (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            requirement_type TEXT NOT NULL,
            value_text TEXT,
            value_numeric REAL,
            units TEXT,
            notes TEXT,
            source TEXT NOT NULL,
            reliability INTEGER,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Ecological interactions (pollinators, pests, symbionts)
    query(r#"
        CREATE TABLE IF NOT EXISTS ecological_interactions (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            interaction_type TEXT NOT NULL,
            related_taxon TEXT,
            notes TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Plant uses (medicinal, ornamental, etc.)
    query(r#"
        CREATE TABLE IF NOT EXISTS uses (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            use_category TEXT NOT NULL,
            description TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Media assets (photos, illustrations)
    query(r#"
        CREATE TABLE IF NOT EXISTS media (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            media_type TEXT NOT NULL,
            url TEXT NOT NULL,
            attribution TEXT,
            license TEXT,
            captured_at TEXT,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // Provenance + licensing per ingested record
    query(r#"
        CREATE TABLE IF NOT EXISTS provenance (
            id TEXT PRIMARY KEY,
            species_id TEXT NOT NULL,
            source TEXT NOT NULL,
            source_record_id TEXT,
            license TEXT,
            retrieved_at TEXT NOT NULL DEFAULT (datetime('now')),
            hash TEXT,
            FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
        )
    "#)
    .execute(pool)
    .await?;

    // FTS virtual table for species & vernacular names (to be populated post-ingestion)
    query(r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS species_name_fts USING fts5(
            species_id UNINDEXED,
            name,
            language_code,
            tokenize='unicode61'
        )
    "#)
    .execute(pool)
    .await?;

    // Indexes to accelerate common lookups
    for idx_sql in [
        // Synonyms & names
        "CREATE INDEX IF NOT EXISTS idx_synonyms_species ON synonyms(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_vernacular_species ON vernacular_names(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_vernacular_lang ON vernacular_names(language_code)",
        // Distribution & traits
        "CREATE INDEX IF NOT EXISTS idx_distribution_species ON distribution_regions(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_distribution_region ON distribution_regions(region_code)",
        "CREATE INDEX IF NOT EXISTS idx_traits_species ON traits(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_traits_name ON traits(trait_name)",
        // Seasonal & cultivation
        "CREATE INDEX IF NOT EXISTS idx_seasonal_species ON seasonal_characteristics(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultreq_species ON cultivation_requirements(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_cultreq_type ON cultivation_requirements(requirement_type)",
        // Uses & media
        "CREATE INDEX IF NOT EXISTS idx_uses_species ON uses(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_media_species ON media(species_id)",
        // Provenance
        "CREATE INDEX IF NOT EXISTS idx_provenance_species ON provenance(species_id)",
        "CREATE INDEX IF NOT EXISTS idx_provenance_source ON provenance(source)"
    ] {
        query(idx_sql).execute(pool).await?;
    }

    Ok(())
}