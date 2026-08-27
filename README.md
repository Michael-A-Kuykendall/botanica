# Botanica

**Botanica is free forever.** No asterisks, no "free for now," no pivot to paid.

[![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)](https://translifeline.org/) [![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)](https://www.thetrevorproject.org/) [![Sponsor](https://img.shields.io/badge/%E2%9D%A4%EF%B8%8F-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

💝 Support Botanica: [sponsor on GitHub](https://github.com/sponsors/Michael-A-Kuykendall) — 100% of support keeps it free forever. See [SPONSORS.md](SPONSORS.md). 🙏

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988

---

<div align="center">
  <img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/botanica/main/assets/botanica-logo.png" alt="Botanica" width="320" height="auto" />
</div>

**Cultivated-plant knowledge base (Rust + DuckDB) — with a loaded public seed.**  
Open-source “straw” for human agricultural / garden plant data: taxonomy, traits, names, provenance. The **schema for personal inventory (L3) is empty by design**; the **knowledge tables are filled** and shipped as parquet. Apps (e.g. [Budsy](https://github.com/Michael-A-Kuykendall/budsy)) write L3 locally.

| | |
|--|--|
| **Status** | **Data loaded** — cultivated KEEP product on GitHub (`data-v0.2.0`) |
| **Product species (KEEP)** | **~18,566** cultivated/germplasm-linked taxa |
| **Warehouse** | ~62.5k USDA Species rows (reference; not the public product) |
| **EN common names (KEEP)** | **~91%** of KEEP |
| **Care-rich core** | **~2.1k** with ≥3 practical Tier1 fields (soil/moisture/height/toxicity) |
| **Engine** | DuckDB locally; **shareable source of truth = parquet tables** |
| **License** | MIT OR Apache-2.0 (code); data PD / CC BY / CC0 — see MANIFEST |
| **Release** | [data-v0.2.0](https://github.com/Michael-A-Kuykendall/botanica/releases/tag/data-v0.2.0) |

Botanica stays free/open for the **knowledge + schema**. Product UX (camera, plant ID, sync) lives in Budsy.

**Open source, not open contribution.** Sole developer: Michael A. Kuykendall. Unsolicited PRs are closed by default. See [CONTRIBUTING.md](CONTRIBUTING.md) and [GOVERNANCE.md](GOVERNANCE.md). Optional support: [SPONSORS.md](SPONSORS.md).

## This is a loaded database, not an empty shell

| Layer | Content | Public seed |
|-------|---------|-------------|
| **L1** Taxonomy | families, genera, species, external IDs | **Yes — filled** |
| **L2** Knowledge | traits, cultivation requirements, vernaculars, synonyms, distribution, provenance | **Yes — filled** (depth varies) |
| **L3** Inventory | *your* plants, photos, care logs | **Schema only (0 rows)** |

### What’s in the public KEEP product right now

| Metric | Value |
|--------|------:|
| Species | ~18,566 |
| Families / genera | ~532 / ~3,687 |
| Vernacular name rows | ~247k |
| Trait rows | ~26k |
| Cultivation requirement rows | ~26k |
| Synonyms | ~35k |
| English vernacular coverage | ~91% of KEEP |
| Practical Tier1 (≥3 of soil/moisture/height/toxicity) | ~11% of KEEP (~2.1k deep core) |
| Hardiness (Wikidata free pass) | ~545 species |
| Uses / cultivars | 0 (not loaded yet) |

**KEEP membership rule:** hort payload (traits / cult.req / uses) **or** GRIN / FAOSTAT allowlist hit.  
Full USDA wild bulk is **not** the product — ~44k empty rows were filtered out.

Sources already in: **USDA PLANTS**, **GRIN taxonomy**, **FAOSTAT** crop labels, **POWO**, **GBIF** vernaculars, **Wikidata** hardiness (sparse).

## Get the data (GitHub columnar)

One **directory per table** under `data/silver_keep/` — each table split into small
`part-NNNNNN.parquet` files (every part ≤ 40 MB, so we never need Git LFS). DuckDB reads
a whole table by globbing the parts. See `docs/DATA_PARQUET.md`.

| Table dir (examples) | ~Total |
|-----------------|------:|
| `vernacular_names/` | ~16 MB |
| `distribution_regions/` | ~57 MB |
| `species_identifiers/` | ~16 MB |
| `species/` | ~4 MB |
| `traits/` / `cultivation_requirements/` | ~1 MB each |
| **All KEEP tables together** | **~126 MB** |

```bash
# after clone
duckdb -c "SELECT count(*) FROM read_parquet('data/silver_keep/species/*.parquet');"
duckdb -c "SELECT scientific_name FROM read_parquet('data/silver_keep/species/*.parquet') WHERE scientific_name ILIKE 'Monstera%' LIMIT 10;"
```

Or download the [Release asset](https://github.com/Michael-A-Kuykendall/botanica/releases/tag/data-v0.2.0).

Rebuild KEEP after fills:

```bash
python scripts/export_keep_set.py --tag <sprint>
```


## Provenance (where data comes from)

Every knowledge fact is labeled by source. See **[\docs/PROVENANCE.md\](docs/PROVENANCE.md)** for licenses, tables, and queries.

| Source | License | Role |
|--------|---------|------|
| USDA PLANTS | Public domain | Taxonomy + most care traits |
| GRIN | Free + attribution | Cultivated / germplasm membership |
| FAOSTAT | Free + attribution | Commercial crop signal |
| POWO (Kew) | CC BY 4.0 | Synonyms, distribution, lifeform/climate |
| GBIF | CC BY 4.0 | Vernacular names |
| Wikidata | CC0 | Sparse hardiness |

Row-level: \provenance\ + \source\ columns on L2 tables. Batch-level: \data/manifests/*.json\.

## Iterative updates (houseplants, crops, …)

See **[`docs/ITERATIVE_FILL.md`](docs/ITERATIVE_FILL.md)**.

```bash
# What’s missing from a priority list (starter houseplants included)?
python scripts/gap_report.py
# → data/manifests/gap-houseplants.txt  then scrape only those gaps
```

Loop: **score → gap list → fail-fast scrape → merge → export KEEP → quality JSON → commit parquets**.

## What works in code today

- Family → genus → species types and CRUD
- DuckDB migrations for L1/L2 + empty L3
- Seed build / KEEP export scripts
- Optional network ingest feature (`--features ingestion`)

## Gaps / not yet

- Deep care fields on most of the 18.5k (only ~2k rich)
- Hardiness / sunlight completeness
- Uses, cultivars
- Some common houseplants still missing from KEEP (see `gap_report.py`)

## Docs

| Doc | Topic |
|-----|--------|
| [`data/README.md`](data/README.md) | Artifacts + load |
| [`docs/ITERATIVE_FILL.md`](docs/ITERATIVE_FILL.md) | Agile fill loop |
| [`docs/RELEASE_PROCESS.md`](docs/RELEASE_PROCESS.md) | Tagging / Release |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Design |
| [`docs/WORKSTREAMS.md`](docs/WORKSTREAMS.md) | Work queue map |

## Quick start (Rust API)

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

Prefer **parquet KEEP** for real species data; the snippet above only demos the in-memory API.

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
