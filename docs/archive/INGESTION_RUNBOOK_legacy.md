# Ingestion Runbook

## Quick Summary
Load plant data from POWO, GBIF, and USDA into Botanica database. All sources are public-domain or open-licensed.

## Build
```bash
cd botanica
cargo build --features ingestion --release
```
Binary: `target/release/ingest`

## Commands

### Bulk load: Seed all cultivated species
```bash
ingest <db.sqlite> bulk-load [max_species]
```
Populates species table from curated list of cultivated plants. Optional: limit to first N species.

Result: Creates family → genus → species hierarchy. Stores POWO IDs for reference.

### POWO: Synonyms, distribution, uses
```bash
ingest <db.sqlite> powo <species_id> <powo_id>
```
Example: `ingest /data/plants.db powo urn:lsid:ipni.org:names:30000023-2`

License: CC BY 4.0  
Citation: "© Kew Science – Plants of the World Online"

### GBIF: Vernacular names (multi-language)
```bash
ingest <db.sqlite> gbif <species_id> <gbif_id>
```
Example: `ingest /data/plants.db gbif 2885675`

License: CC BY 4.0  
Citation: "GBIF.org Vernacular Names"

### USDA: Traits and tolerances
```bash
ingest <db.sqlite> usda-csv <csv_path>
```
Example: `ingest /data/plants.db usda-csv species_characteristics.csv`

CSV columns (required): symbol, growth_habit, duration, mature_height_m, drought_tolerance, shade_tolerance, salinity_tolerance, wetland_indicator

License: Public Domain  
Citation: "USDA PLANTS Database"

### Rebuild full-text search
```bash
ingest <db.sqlite> fts-rebuild
```
Run after any ingestion to ensure name search works.

## Trait Normalization
Raw data is normalized to controlled vocab:

**growth_habit**: tree, shrub, herb, vine, grass, sedge, fern, succulent, other  
**duration**: annual, biennial, perennial, other  
**drought_tolerance**: none, low, medium, high, unknown  
**shade_tolerance**: none, low, medium, high, unknown  
**salinity_tolerance**: none, low, medium, high, unknown  
**wetland_indicator**: upland, facultative_upland, facultative, facultative_wetland, obligate_wetland, unknown  

Height: numeric in meters (2 decimal precision).

## What Gets Stored
- **species**: Scientific names (loaded externally, not by ingester).
- **synonyms**: POWO accepted names + authored names.
- **distribution_regions**: WGSRPD region codes.
- **uses**: Use categories + descriptions.
- **vernacular_names**: Language-tagged common names.
- **traits**: Text or numeric values with units.
- **cultivation_requirements**: Tolerance levels and indicators.
- **provenance**: Source, timestamp, SHA-256 hash of payload (audit trail).

## Error Handling
- Missing species_id: skipped with warning; add species first.
- Malformed CSV rows: logged and skipped; import continues.
- Network failure: error halts import (retry or check connectivity).
- Duplicate data: INSERT OR IGNORE silently prevents duplicates.

## Environment Overrides
```bash
POWO_BASE_URL=https://powo.science.kew.org/api/2 \
GBIF_BASE_URL=https://api.gbif.org/v1 \
USDA_BASE_URL=https://plantsdb.xyz/api \
ingest /data/plants.db powo species123 powo_id
```

## Verification
Query FTS search after import:
```bash
sqlite3 /data/plants.db "SELECT * FROM species_name_fts WHERE scientific_name MATCH 'rosa' LIMIT 5;"
```

## Notes
- Hashes prevent accidental re-imports of identical payloads.
- All times stored in UTC (datetime('now')).
- Case-insensitive trait matching (raw values lowercased before mapping).
- Heights >10000m or <0m rejected (data quality filter).
