# Name resolution policy (Phase 3)

How botanica turns source names into L1 taxonomy without inventing fake taxa.

## Rules

1. **Prefer external IDs** (`species_identifiers`) for merge: `usda` / `powo` / `gbif` / …
2. **Parse binomial** from scientific name: first token = genus, second = specific epithet. Authority suffixes (`L.`, `Mill.`, parentheticals) are stripped for storage of `scientific_name`.
3. **Family resolution**
   - If master/CSV provides `family` → use it.
   - Else look up genus in `data/lookups/genus_family.csv`.
   - Else **quarantine** (`ingest_quarantine`, reason `missing_family` / `missing_family_lookup`).  
     Do **not** insert `Unknown` family/genus rows.
4. **Unparseable names** → quarantine (`unparseable_binomial`).
5. **Idempotency**: same `(source, external_id)` is skipped on re-ingest.
6. **Taxonomic status**: new seed rows default to `accepted` / rank `species` until synonym pipelines exist.

## Gate2 pilot

- Bronze: `data/bronze/gate2/USDA_PLANTS_norm.json` (from Budsy USDA scrape)
- Lookup: `data/lookups/genus_family.csv` (curated for genera in that pilot)
- Empty / null scientific names in bronze → quarantine

## Expanding the cultivated corpus

1. Grow `genus_family.csv` (or replace with a better backbone later).
2. Prefer master lists that ship `family,genus,scientific_name,symbol`.
3. Optional later: WFO/POWO resolution for missing families (network path under `ingestion` feature).
