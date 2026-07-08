# Botanica

**Cultivated-plant knowledge base (Rust + DuckDB).**  
Open-source “straw” for human agricultural / garden plant data — taxonomy, traits, names, provenance. Personal inventory tables ship **empty**; apps (e.g. [Budsy](https://github.com/Michael-A-Kuykendall/budsy)) write those locally.

| | |
|--|--|
| **Status** | Active rebuild — schema + ingest in progress; not a finished world flora |
| **Engine** | DuckDB (not SQLite) |
| **License** | MIT OR Apache-2.0 |
| **Architecture** | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |

Botanica stays free/open for the **knowledge + schema**. Product UX (camera, plant ID, sync) lives in Budsy.

## What works today

- Family → genus → species types and CRUD
- DuckDB migrations for taxonomy, horticultural reference, and **empty** inventory tables
- Optional ingest feature: POWO / GBIF / USDA scaffolding (`--features ingestion`)
- Tests for core taxonomy paths

## What does *not* work (yet) / removed

- Full cultivated world seed (parquet silver + load) — **planned**, see architecture phases 3–5
- ContextLite / “AI insights” — **removed**
- Marketing claims of “production-ready institutional use” — **retired**; this README is the truth source
- Feature flags `herbarium` / `germplasm` / `api` — flags only; no real modules yet (do not enable expecting magic)

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
