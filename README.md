# Botanica

**Cultivated-plant knowledge base (Rust + DuckDB).**  
Open-source “straw” for human agricultural / garden plant data — taxonomy, traits, names, provenance. Personal inventory tables ship **empty**; apps (e.g. [Budsy](https://github.com/Michael-A-Kuykendall/budsy)) write those locally.

| | |
|--|--|
| **Status** | Cultivated **KEEP** silver (~3k species with care payload) + full warehouse; public product is columnar parquet on GitHub |
| **Engine** | DuckDB (not SQLite); shareable source of truth is **parquet** |
| **License** | MIT OR Apache-2.0 (crate); data sources PD / CC BY — see MANIFEST |
| **Architecture** | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · workstreams [`docs/WORKSTREAMS.md`](docs/WORKSTREAMS.md) |

Botanica stays free/open for the **knowledge + schema**. Product UX (camera, plant ID, sync) lives in Budsy.

**Open source, not open contribution.** Sole developer: Michael A. Kuykendall. Unsolicited PRs are closed by default. See [CONTRIBUTING.md](CONTRIBUTING.md) and [GOVERNANCE.md](GOVERNANCE.md). Optional support: [SPONSORS.md](SPONSORS.md).

## Public data product (what the world should use)

| Path | What |
|------|------|
| **`data/silver_keep/*.parquet`** | Cultivated KEEP — species with hort/care payload |
| `data/manifests/botanica-keep-baseline.json` | Counts + file list |
| `data/manifests/quality-keep-baseline.json` | Coverage on KEEP only |
| `data/manifests/keep-membership.json` | Filter rule + keep/drop counts |

**KEEP rule:** has `traits` OR `cultivation_requirements` OR `uses`.
Current baseline: **~18,566 keep** / ~62.5k warehouse (GRIN+FAOSTAT+payload) KEEP parquet (fits GitHub).

### Load in one command

```bash
duckdb -c "SELECT count(*) FROM read_parquet('data/silver_keep/species.parquet');"
```

Rebuild KEEP after warehouse updates:

```bash
python scripts/export_keep_set.py --tag baseline
```

Details: [`data/README.md`](data/README.md) · release: [`docs/RELEASE_PROCESS.md`](docs/RELEASE_PROCESS.md)

## What works today

- Family → genus → species types and CRUD
- DuckDB migrations for L1/L2 knowledge + empty L3 inventory
- USDA taxonomy warehouse + HasChar traits + POWO/GBIF enrich
- **KEEP export** for cultivated/ag public slice
- Optional ingest feature: POWO / GBIF / USDA scaffolding (`--features ingestion`)

## What does *not* work (yet) / removed

- Global GRIN/FAOSTAT allowlists (workstream B — expands KEEP)
- Hardiness/sunlight depth on KEEP (workstream C)
- Cultivars (0 rows until free source)
- ContextLite / AI insights — **removed**
- Feature flags `herbarium` / `germplasm` / `api` — incomplete stubs

## Quick start

```toml
[dependencies]
botanica = { version = "0.3", path = "..." }  # or crates.io when published truthfully
tokio = { version = "1", features = ["full"] }
```

```rust
use botanica::{BotanicalDatabase, Family, Genus, Species};
use botanica::queries::{family, genus, species};

#[tokio::main]
async fn main() -> botanica::Result<()> {
    let db = BotanicalDatabase::memory().await?;
    db.migrate().await?;

    let rosaceae = Family::new("Rosaceae".into(), "Juss.".into());
    family::insert_family(&db, &rosaceae).await?;

    let rosa = Genus::new(rosaceae.id, "Rosa".into(), "L.".into());
    genus::insert_genus(&db, &rosa).await?;

    let briar = Species::new(
        rosa.id,
        "rubiginosa".into(),
        "L.".into(),
        Some(1753),
        Some("LC".into()),
    );
    species::insert_species(&db, &briar).await?;

    let found = family::get_families_by_name(&db, "Rosaceae").await?;
    println!("families: {}", found.len());
    Ok(())
}
```

## Data layers (short)

| Layer | Content | In OSS seed |
|-------|---------|-------------|
| L1 Taxonomy | family / genus / species / cultivars | Yes |
| L2 Knowledge | traits, names, uses, provenance | Yes |
| L3 Inventory | *your* plants, photos, care, health | Schema only (0 rows) |

Full model, phases, and decisions: **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)**.

## Build seed (gate2 pilot)

```bash
cargo run --bin build_seed
# → data/silver/*.parquet + data/manifests/botanica-cultivated-v0.1.json
# see data/README.md and docs/NAME_RESOLUTION.md
```

## Ingest (optional network importers)

```bash
cargo build --features ingestion --release
# see docs/RUNBOOK_INGEST.md (DuckDB paths; no sqlite3)
```

## Features

| Feature | Purpose |
|---------|---------|
| `ingestion` | HTTP/CSV importers + CLI bins |
| `darwin-core` | DwC types (partial; not a full GBIF stack) |
| `conservation` | IUCN types (mock client — not production) |
| `full` | Turns on optional pro flags (still incomplete) |

Default features: **none**.

## Related

- **Budsy** — app lifecycle, inventory UI, CrabCamera
- **CrabCamera** — desktop capture (plant-agnostic)

## License

MIT OR Apache-2.0. Knowledge stays open. Build cool things.
