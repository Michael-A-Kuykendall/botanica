/// Database schema definitions

/// SQL for the families table
pub const FAMILIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS families (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    authority TEXT
)
"#;

/// SQL for the genera table
pub const GENERA_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS genera (
    id TEXT PRIMARY KEY,
    family_id TEXT NOT NULL,
    name TEXT NOT NULL,
    authority TEXT,
    FOREIGN KEY (family_id) REFERENCES families(id)
)
"#;

/// SQL for the species table
pub const SPECIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS species (
    id TEXT PRIMARY KEY,
    genus_id TEXT NOT NULL,
    specific_epithet TEXT NOT NULL,
    authority TEXT NOT NULL,
    publication_year INTEGER,
    conservation_status TEXT,
    FOREIGN KEY (genus_id) REFERENCES genera(id)
)
"#;

/// SQL for user's individual plants (instances of species)
pub const PLANTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS plants (
    id TEXT PRIMARY KEY,
    species_id TEXT,
    user_given_name TEXT NOT NULL,
    acquired_date TEXT,
    location TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (species_id) REFERENCES species(id)
)
"#;

/// SQL for plant photos with AI analysis results
pub const PLANT_PHOTOS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS plant_photos (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    taken_at TEXT NOT NULL DEFAULT (datetime('now')),
    ai_analysis_json TEXT,
    notes TEXT,
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
)
"#;

/// SQL for cultivation/care activity logs
pub const CARE_ACTIVITIES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS care_activities (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    notes TEXT,
    photo_id TEXT,
    performed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (photo_id) REFERENCES plant_photos(id)
)
"#;

/// SQL for environmental readings over time
pub const ENVIRONMENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY,
    plant_id TEXT,
    temperature_celsius REAL,
    humidity_percent REAL,
    ph_level REAL,
    light_hours REAL,
    co2_ppm INTEGER,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
)
"#;

/// SQL for cultivation records tracking growth stages
pub const CULTIVATION_RECORDS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cultivation_records (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    growth_stage TEXT NOT NULL,
    environment_id TEXT,
    notes TEXT,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    cultivator TEXT,
    FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE,
    FOREIGN KEY (environment_id) REFERENCES environments(id)
)
"#;

/// SQL for horticultural reference: synonyms of accepted species names
pub const SYNONYMS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: vernacular (common) names
pub const VERNACULAR_NAMES_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: geographic distribution regions (e.g. WGSRPD codes)
pub const DISTRIBUTION_REGIONS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: species traits (structured + free text)
pub const TRAITS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: seasonal characteristics (flowering, fruiting, dormancy)
pub const SEASONAL_CHARACTERISTICS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: cultivation requirements (light, soil, pH, moisture)
pub const CULTIVATION_REQUIREMENTS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: ecological interactions (pollinators, pests, symbionts)
pub const ECOLOGICAL_INTERACTIONS_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: plant uses (medicinal, ornamental, culinary, industrial)
pub const USES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS uses (
    id TEXT PRIMARY KEY,
    species_id TEXT NOT NULL,
    use_category TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (species_id) REFERENCES species(id) ON DELETE CASCADE
)
"#;

/// SQL for horticultural reference: media assets (photos, illustrations)
pub const MEDIA_TABLE_SQL: &str = r#"
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
"#;

/// SQL for horticultural reference: provenance + licensing per ingested record
pub const PROVENANCE_TABLE_SQL: &str = r#"
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
"#;

/// SQL for FTS virtual table aggregating species names (scientific + vernacular + synonyms)
pub const SPECIES_NAME_FTS_SQL: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS species_name_fts USING fts5(
    species_id UNINDEXED,
    name,
    language_code,
    tokenize='unicode61'
)
"#;