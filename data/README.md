# Botanica data artifacts

| Path | Role |
|------|------|
| `bronze/` | Raw source payloads (local; large dumps may be gitignored) |
| `lookups/` | Curated maps (e.g. genus → family) |
| `silver/` | Full warehouse parquet (L1+L2; includes non-cultivated taxonomy bulk) |
| `gold/` | **Gold curation mart** — `species_curation.parquet` (definitive signal score + `is_definitive`) |
| **`silver_keep/`** | **Public product** — cultivated KEEP, driven by `gold/species_curation.is_definitive` |
| `manifests/` | MANIFEST + quality + keep-membership json |
| `botanica-cultivated-v*.duckdb` | Optional local engine (usually gitignored) |

**GitHub packaging:** KEEP set is ~**20 MB** total across parquet files (each file = one table). Full warehouse ~**60 MB**. Both under normal GitHub file limits; Release zip optional for convenience.

## KEEP filter (product slice)

```bash
python scripts/export_keep_set.py --tag baseline
# → data/silver_keep/*.parquet
# → data/manifests/keep-membership.json
# → data/manifests/quality-keep-baseline.json
# → data/manifests/botanica-keep-baseline.json
```

Rule: **KEEP** = species with ≥1 `traits` OR `cultivation_requirements` OR `uses` row. Everything else drops from the public product.

## Load KEEP into DuckDB (anyone)

```bash
# after clone
duckdb -c "SELECT count(*) FROM read_parquet('data/silver_keep/species.parquet');"
duckdb -c "SELECT scientific_name FROM read_parquet('data/silver_keep/species.parquet') LIMIT 10;"
```

Or in Python:

```python
import duckdb
con = duckdb.connect()
print(con.execute(
    "SELECT count(*) FROM read_parquet('data/silver_keep/species.parquet')"
).fetchone())
```

## Build full warehouse seed

```bash
cargo run --release --bin build_seed -- usda
python scripts/export_keep_set.py --tag baseline
```

L3 `plants` count must be **0** in the OSS seed.

## License notes

USDA PLANTS–derived data is **public domain**. POWO/GBIF enrichments are **CC BY 4.0** — attribution in MANIFEST/provenance.
