# Parquet data layout & size convention

**Rule:** every parquet table ships as a **directory of small part files** that DuckDB
reads by glob, so **no single file ever approaches Git / GitHub's large-file limits**.

* **Layout:** `<dir>/<table>/part-NNNNNN.parquet` (e.g. `data/silver_keep/species/part-000000.parquet`)
* **Target:** every part ≤ **40 MB** (`TARGET_MB_DEFAULT` in `scripts/shard_parquet.py`)
* **Why 40 MB:** GitHub warns above 50 MB and hard-rejects above 100 MB. 40 MB leaves
  comfortable headroom so a part never crosses the warning even as it grows, and we
  **never need Git LFS**.
* **Reading a whole table:** `read_parquet('data/silver_keep/<table>/*.parquet')`

## Why

GitHub's large-file policy (warn ≥ 50 MB, reject ≥ 100 MB) plus Git LFS quotas make big
single parquet blobs a liability. A table split into many small parts is **just as
usable** — DuckDB's `read_parquet()` globs the parts transparently — and keeps the
repository cloneable and the data relocatable. This was the design intent from the
start; the sharder just enforces it mechanically.

## Tools

```bash
# Normalize data/silver_keep and data/silver: split any oversize/flat tables into parts
python scripts/shard_parquet.py --data data/silver_keep data/silver

# CI gate: fail (exit 1) if any part is over the target
python scripts/shard_parquet.py --data data/silver_keep data/silver --verify-only
```

## Writers must shard

Any script that **writes** parquet must route the output through the sharder so the
convention holds automatically. Current writers already do this:

* `scripts/export_keep_set.py` → shards `data/silver_keep`
* `scripts/ingest_wcvp.py` / `scripts/ingest_grin_backbone.py` / `scripts/ingest_faostat_crops.py`
  → shards `data/silver`
* `src/seed/export.rs` (`export_silver_parquet`) writes directly into
  `<dir>/<table>/part-000000.parquet`

**New writer rule:** write the flat staging parquet, then call
`shard_out(<dir>, TARGET_MB_DEFAULT, con)` (import from `shard_parquet`) before finishing.

## Readers

DuckDB reads a table via the directory glob:

```sql
SELECT count(*) FROM read_parquet('data/silver_keep/species/*.parquet');
```

Legacy flat files should not be referenced; `read_parquet('<dir>/<table>.parquet')` no
longer exists after sharding.

## Loader

`src/seed/export.rs::load_silver_parquet` reads each table from `<dir>/<table>/*.parquet`
(with a flat-file fallback for backward compatibility) and repopulates the DuckDB tables.

## CI

`.github/workflows/ci.yml` runs `scripts/shard_parquet.py --verify-only` so any commit
that pushes a part over the target fails the build before it can reach `main`.