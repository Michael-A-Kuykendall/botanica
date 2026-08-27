# Ingestion runbook (DuckDB)

Truth-aligned with current code. Architecture: [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Build

```bash
cargo build --features ingestion --release
```

Binaries: `target/release/ingest`, `target/release/discover`

## Commands

```text
ingest <db_path> powo <species_id> <powo_id>
ingest <db_path> gbif <species_id> <gbif_id>
ingest <db_path> usda <species_id> <usda_key>
ingest <db_path> usda-csv <csv_path>
ingest <db_path> fts-rebuild    # currently a no-op (search uses ILIKE)
ingest <db_path> perf
ingest <db_path> bulk-load [max_species]
```

`db_path` is a **DuckDB** file path (e.g. `plants.duckdb`), not SQLite.

## Status (honest)

| Path | Status |
|------|--------|
| Per-species POWO/GBIF/USDA | Scaffolding present; validate against live APIs before relying |
| `usda-csv` | Preferred bulk path for traits |
| `bulk-load` without master list | Demo list only — do not treat as full cultivated load |
| Parquet silver + `load_seed` | **Planned** (architecture Phase 3) |
| FTS rebuild | No-op on DuckDB today |

## Environment overrides

```bash
POWO_BASE_URL=...
GBIF_BASE_URL=...
USDA_BASE_URL=...
```

## Licenses / attribution

Store source, license, and retrieval metadata in `provenance` (and per-row `source` fields). Do not strip attribution on re-export.

## Verification

Open the DuckDB file with the DuckDB CLI or a small Rust query — not `sqlite3`.

```sql
SELECT COUNT(*) FROM species;
SELECT * FROM provenance LIMIT 5;
```
