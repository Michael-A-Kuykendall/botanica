/// Database schema definitions for DuckDB
/// Schema version: 0.4.0 (Phase 2 — identifiers, cultivars, quarantine, inventory harden)

pub const SCHEMA_VERSION: &str = "0.4.0";

/// Schema metadata (single-row style version stamp)
pub const SCHEMA_META_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_meta (
    key VARCHAR PRIMARY KEY,
    value VARCHAR NOT NULL
)
"#;

/// SQL for the families table
pub const FAMILIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS families (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    authority VARCHAR
)
"#;

/// SQL for the genera table
pub const GENERA_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS genera (
    id VARCHAR PRIMARY KEY,
    family_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    authority VARCHAR,
    FOREIGN KEY (family_id) REFERENCES families(id)
)
"#;

/// SQL for the species table
pub const SPECIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS species (
    id VARCHAR PRIMARY KEY,
    genus_id VARCHAR NOT NULL,
    specific_epithet VARCHAR NOT NULL,
    authority VARCHAR NOT NULL,
    publication_year INTEGER,
    conservation_status VARCHAR,
    scientific_name VARCHAR,
    taxonomic_status VARCHAR NOT NULL DEFAULT 'accepted',
    rank VARCHAR NOT NULL DEFAULT 'species',
    FOREIGN KEY (genus_id) REFERENCES genera(id)
)
"#;

/// External identifiers for merge across USDA / POWO / GBIF / etc.
/// Unique (source, external_id) enforced via index in runner (portable).
pub const SPECIES_IDENTIFIERS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS species_identifiers (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    source VARCHAR NOT NULL,
    external_id VARCHAR NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// Cultivars / trade names under a species
pub const CULTIVARS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cultivars (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    cultivar_name VARCHAR NOT NULL,
    trade_name VARCHAR,
    source VARCHAR,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// Unresolved ingest rows (no Unknown taxa pollution)
pub const INGEST_QUARANTINE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS ingest_quarantine (
    id VARCHAR PRIMARY KEY,
    source VARCHAR NOT NULL,
    external_id VARCHAR,
    raw_name VARCHAR,
    reason VARCHAR NOT NULL,
    payload_hash VARCHAR,
    created_at TIMESTAMP DEFAULT current_timestamp
)
"#;

/// User's individual plants (one row per individual)
pub const PLANTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS plants (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR,
    cultivar_id VARCHAR,
    user_given_name VARCHAR NOT NULL,
    health_status VARCHAR NOT NULL DEFAULT 'unknown',
    acquired_date VARCHAR,
    location VARCHAR,
    notes VARCHAR,
    user_id VARCHAR,
    device_id VARCHAR,
    created_at TIMESTAMP DEFAULT current_timestamp,
    updated_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id),
    FOREIGN KEY (cultivar_id) REFERENCES cultivars(id)
)
"#;

/// SQL for plant photos with AI analysis results
pub const PLANT_PHOTOS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS plant_photos (
    id VARCHAR PRIMARY KEY,
    plant_id VARCHAR NOT NULL,
    file_path VARCHAR NOT NULL,
    taken_at TIMESTAMP DEFAULT current_timestamp,
    ai_analysis_json VARCHAR,
    notes VARCHAR,
    FOREIGN KEY (plant_id) REFERENCES plants(id)
)
"#;

/// SQL for cultivation/care activity logs
pub const CARE_ACTIVITIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS care_activities (
    id VARCHAR PRIMARY KEY,
    plant_id VARCHAR NOT NULL,
    activity_type VARCHAR NOT NULL,
    notes VARCHAR,
    photo_id VARCHAR,
    performed_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (plant_id) REFERENCES plants(id),
    FOREIGN KEY (photo_id) REFERENCES plant_photos(id)
)
"#;

/// SQL for environmental readings over time
pub const ENVIRONMENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS environments (
    id VARCHAR PRIMARY KEY,
    plant_id VARCHAR,
    temperature_celsius DOUBLE,
    humidity_percent DOUBLE,
    ph_level DOUBLE,
    light_hours DOUBLE,
    co2_ppm INTEGER,
    recorded_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (plant_id) REFERENCES plants(id)
)
"#;

/// SQL for cultivation records tracking growth stages
pub const CULTIVATION_RECORDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cultivation_records (
    id VARCHAR PRIMARY KEY,
    plant_id VARCHAR NOT NULL,
    growth_stage VARCHAR NOT NULL,
    environment_id VARCHAR,
    notes VARCHAR,
    recorded_at TIMESTAMP DEFAULT current_timestamp,
    cultivator VARCHAR,
    FOREIGN KEY (plant_id) REFERENCES plants(id),
    FOREIGN KEY (environment_id) REFERENCES environments(id)
)
"#;

/// SQL for horticultural reference: synonyms of accepted species names
pub const SYNONYMS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS synonyms (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    synonym_name VARCHAR NOT NULL,
    authorship VARCHAR,
    source VARCHAR NOT NULL,
    source_record_id VARCHAR,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: vernacular (common) names
pub const VERNACULAR_NAMES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS vernacular_names (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    language_code VARCHAR,
    is_primary INTEGER NOT NULL DEFAULT 0,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: geographic distribution regions (e.g. WGSRPD codes)
pub const DISTRIBUTION_REGIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS distribution_regions (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    region_code VARCHAR NOT NULL,
    region_source VARCHAR,
    notes VARCHAR,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: species traits (structured + free text)
pub const TRAITS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS traits (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    trait_name VARCHAR NOT NULL,
    trait_value_text VARCHAR,
    trait_value_numeric DOUBLE,
    units VARCHAR,
    method VARCHAR,
    source VARCHAR NOT NULL,
    reliability INTEGER,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: seasonal characteristics (flowering, fruiting, dormancy)
pub const SEASONAL_CHARACTERISTICS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS seasonal_characteristics (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    season VARCHAR NOT NULL,
    characteristic_type VARCHAR NOT NULL,
    value VARCHAR,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: cultivation requirements (light, soil, pH, moisture)
pub const CULTIVATION_REQUIREMENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cultivation_requirements (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    requirement_type VARCHAR NOT NULL,
    value_text VARCHAR,
    value_numeric DOUBLE,
    units VARCHAR,
    notes VARCHAR,
    source VARCHAR NOT NULL,
    reliability INTEGER,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: ecological interactions (pollinators, pests, symbionts)
pub const ECOLOGICAL_INTERACTIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS ecological_interactions (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    interaction_type VARCHAR NOT NULL,
    related_taxon VARCHAR,
    notes VARCHAR,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: plant uses (medicinal, ornamental, culinary, industrial)
pub const USES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS uses (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    use_category VARCHAR NOT NULL,
    description VARCHAR,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: media assets (photos, illustrations)
pub const MEDIA_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS media (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    media_type VARCHAR NOT NULL,
    url VARCHAR NOT NULL,
    attribution VARCHAR,
    license VARCHAR,
    captured_at TIMESTAMP,
    source VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT current_timestamp,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for horticultural reference: provenance + licensing per ingested record
pub const PROVENANCE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS provenance (
    id VARCHAR PRIMARY KEY,
    species_id VARCHAR NOT NULL,
    source VARCHAR NOT NULL,
    source_record_id VARCHAR,
    license VARCHAR,
    retrieved_at TIMESTAMP DEFAULT current_timestamp,
    hash VARCHAR,
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// Ordered list of required tables for validate_migrations
pub const REQUIRED_TABLES: &[&str] = &[
    "schema_meta",
    "families",
    "genera",
    "species",
    "species_identifiers",
    "cultivars",
    "ingest_quarantine",
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
    "plants",
    "plant_photos",
    "care_activities",
    "environments",
    "cultivation_records",
];
