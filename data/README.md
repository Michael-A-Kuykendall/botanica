# Botanica data artifacts

| Path | Role |
|------|------|
| `bronze/` | Raw/normalized source payloads (local; may be committed for small pilots) |
| `lookups/` | Curated maps (e.g. genus → family) |
| `silver/` | **Published knowledge** — per-table parquet (L1+L2) |
| `manifests/` | Build MANIFEST json (counts, licenses, schema version) |
| `botanica-cultivated-v*.duckdb` | Optional built engine file (gitignored if large) |

## Build seed (pilot)

From repo root (enough free disk for DuckDB compile):

```bash
cargo run --bin build_seed
```

Produces:

- `data/botanica-cultivated-v0.1.duckdb`
- `data/silver/*.parquet`
- `data/manifests/botanica-cultivated-v0.1.json`

L3 `plants` count must be **0** in the OSS seed.

## Load parquet into a fresh DuckDB

```rust
// sketch — see botanica::seed::export::load_silver_parquet
```

Or CLI later. Architecture: silver parquet is the GitHub-friendly source of truth; DuckDB is local engine.

## License notes

Gate2 bronze is USDA PLANTS–derived (**public domain**). Attribution still recorded in `provenance`.
