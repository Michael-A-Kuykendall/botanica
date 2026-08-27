# Botanica Stack Architecture

**Status:** Draft for audit (Session 0)  
**Canonical home:** `botanica/docs/ARCHITECTURE.md`  
**Mirror:** `budsy/docs/BOTANICA_STACK_ARCHITECTURE.md` (pointer + Budsy slice)  
**Related crates:** `botanica` (this repo), `budsy` (`../budsy`), `crabcamera` (`../crabcamera`)  
**Date:** 2026-07-08  
**Owner:** Michael A. Kuykendall  

---

## 0. Purpose of this document

This is the single architecture for the product family:

| Crate | Role | Business boundary |
|-------|------|-------------------|
| **Botanica** | Open-source cultivated-plant knowledge base + schema + ingest | Free forever: the straw that accumulates human agricultural/horticultural knowledge |
| **Budsy** | Paid application lifecycle: inventory, ID, care, ops | Sells the experience (camera → understand *my* plants) |
| **CrabCamera** | Capture substrate (desktop camera/A-V) | Independent product; Budsy consumes it |

This document is meant to be **audited until sound**, then executed in phases. It is not marketing. Claims here must match code or be labeled **PLANNED**.

---

## 1. Product intent (sifted)

### 1.1 What we are building

A **finite, human-curated plant knowledge system** — not every wild species on Earth.

Scope = plants humans **garden, farm, cultivate, breed, or commercially trade** (crops, ornamentals, herbs, trees in cultivation, cultivars where data exists). That set is large but bounded and sourceable.

### 1.2 What success looks like

1. **Offline knowledge file** anyone can open: taxonomy + traits + names + provenance.
2. **Personal inventory** in the same schema: “I have nine tomatoes in different health states.”
3. **Budsy** uses CrabCamera (and later webcams) to observe *local* plants and bind them to knowledge rows.
4. **Open core:** knowledge + schema + ingest are OSS. App UX, workflows, and commercial packaging live in Budsy.

### 1.3 What we are explicitly *not* doing (now)

| Out of scope now | Why |
|------------------|-----|
| Full global wild flora | Impossible / wrong product |
| Real-time IUCN as a v1 feature | Mock today; not on critical path |
| ContextLite | **Removed** (Q6) — DuckDB/columnar knowledge only |
| Darwin Core / herbarium / germplasm feature flags | Vapor; freeze |
| Dual finished mobile + desktop Budsy | Pick one path after clean park |
| Committing plant photo binaries to git | Size + license landmines |
| Full medallion lakehouse day one | Prepare hooks only |

---

## 2. System context

```
                    ┌──────────────────────────────────────┐
                    │           DATA SOURCES               │
                    │  USDA · POWO · GBIF · (later WFO…)   │
                    └──────────────────┬───────────────────┘
                                       │ ingest (Botanica CLI)
                                       ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ BOTANICA (OSS crate + seed artifact)                                     │
│                                                                          │
│  L1 TAXONOMY     family → genus → species → [cultivar]                   │
│  L2 KNOWLEDGE    traits, names, uses, media URLs, provenance             │
│  L3 INVENTORY    plants, photos, care, env, growth  (schema only in OSS) │
│                                                                          │
│  Artifact: data/botanica-cultivated-vX.Y.duckdb  (L1+L2 filled, L3 empty)│
└───────────────────────────────────┬──────────────────────────────────────┘
                                    │ open / copy seed
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ BUDSY (application)                                                      │
│  - owns app lifecycle, UI, ID orchestration                              │
│  - writes L3 only                                                        │
│  - may call external Plant ID APIs                                       │
│  - later: multi-cam / greenhouse ops                                     │
└───────────────┬──────────────────────────────────────────────────────────┘
                │ capture
                ▼
         ┌─────────────┐
         │ CrabCamera  │  (mature independent crate)
         └─────────────┘
```

### 2.1 Trust boundaries

| Data | Mutability | Redistributable | Lives in OSS seed |
|------|------------|-----------------|-------------------|
| L1 taxonomy | Curated / versioned releases | Yes (with source licenses) | Yes |
| L2 knowledge facts | Append via ingest; immutable rows preferred | Yes if license allows | Yes |
| L3 user inventory | User-local writes | **No** | Schema only, zero rows |
| Photos (files) | Local disk / user storage | No | Never in git |
| Photo metadata | L3 `plant_photos` | No | Empty in seed |

---

## 3. Data layers (canonical model)

Three layers. One schema. Two lifetimes (release knowledge vs user instance).

### 3.1 L1 — Taxonomy (backbone)

**Question:** What is this organism called, scientifically?

| Entity | Purpose |
|--------|---------|
| `families` | Family + authority |
| `genera` | Genus under family |
| `species` | Species under genus |
| `cultivars` *(PLANNED add)* | Cultivar / trade name under species |
| `species_identifiers` *(PLANNED add)* | External IDs: usda_symbol, powo_id, gbif_id, wfo_id… |

**Rules:**

- Never invent `Unknown` family/genus as “truth.” Unresolved hierarchy → quarantine table or null FK with explicit status, not fake taxa.
- Prefer stable external IDs for merge; UUIDs are internal only.
- Denormalized `scientific_name` on species (or a view) for search/join ergonomics.

### 3.2 L2 — Knowledge (the straw)

**Question:** What do humans know about growing/using this taxon?

| Entity | Purpose |
|--------|---------|
| `synonyms` | Alternate scientific names |
| `vernacular_names` | Common names + language |
| `distribution_regions` | Native/cultivated regions (codes) |
| `traits` | Structured key/value (height, habit, …) |
| `seasonal_characteristics` | Flowering, fruiting, dormancy |
| `cultivation_requirements` | Light, soil, moisture, hardiness… |
| `ecological_interactions` | Pests, pollinators (sparse OK) |
| `uses` | Culinary, medicinal, ornamental, industrial |
| `media` | **URLs + attribution + license only** |
| `provenance` | Source, external id, license, retrieved_at, content hash |

**Rules:**

- Every L2 fact row must be attributable via `source` and ideally a `provenance` record.
- Traits use controlled vocabularies where we define them (see normalization module).
- Knowledge is **append-friendly**; corrections are new rows or versioned supersession (document policy when implemented).

### 3.3 L3 — Inventory / operation (Budsy writes)

**Question:** Which physical plants do *I* have, and how are they doing?

| Entity | Purpose |
|--------|---------|
| `plants` | One row = one individual (or one managed unit) |
| `plant_photos` | Local file path + optional AI JSON |
| `care_activities` | Water, feed, prune, treat, observe |
| `environments` | Spot readings (temp, humidity, …) |
| `cultivation_records` | Growth stage timeline |

**Inventory model (decision):**

```
species (L1) ──< plants (L3)     "I have 9 plants of Solanum lycopersicum"
                     │
                     ├── plant_photos
                     ├── care_activities
                     ├── environments
                     └── cultivation_records
```

Nine tomatoes in different health = **nine `plants` rows**, same `species_id`, different `user_given_name` / `health_status` / `location`.

**PLANNED `plants` fields to harden:**

| Field | Notes |
|-------|-------|
| `species_id` / optional `cultivar_id` | Link to knowledge |
| `user_given_name` | “Back porch #3” |
| `health_status` | enum: healthy / stressed / declining / dead / dormant / unknown |
| `location` | Free text v1; `sites` table later for multi-cam ops |
| `acquired_date`, `notes` | Existing |
| `batch_id` | Optional later; not required for v1 |

### 3.4 Why personal data stays in Botanica schema

**Decision:** Do not invent a second schema product for inventory.

- One mental model for Budsy and future tools.
- OSS ships L3 tables **empty**.
- Budsy opens/copies seed DB and writes L3 locally.
- Future option: DuckDB `ATTACH` read-only knowledge + writable user file — same schema, two files. Design does not block that.

---

## 4. Medallion / lakehouse readiness (without building a lakehouse)

You asked to leave the door open for bronze/silver/gold later. Relevant; do **not** implement a full lakehouse now.

### 4.1 Mapping (conceptual)

| Medallion | In this stack | Physical form (now / later) |
|-----------|---------------|-----------------------------|
| **Bronze** | Raw payloads as retrieved | Files under `data/bronze/{source}/{date}/` + hash; optional raw JSON column later |
| **Silver** | Normalized relational L1/L2 | DuckDB tables (current target) |
| **Gold** | Product marts / search views / app-facing projections | SQL views, export parquet, or Budsy-specific caches |

### 4.2 Preparations that cost little and matter later

1. **Provenance + content hash** on every ingest (already intended).
2. **Idempotent ingest** keys: `(source, external_id)` or payload hash.
3. **Partition by source** in bronze paths (filesystem), not premature table explosion.
4. **Stable trait names** + normalization maps (controlled vocab).
5. **Export path** PLANNED: `EXPORT DATABASE` / per-table parquet for warehouse handoff.
6. **No** separate Spark/warehouse stack until silver is full and you have a real analytics consumer.

### 4.3 What *not* to do

- Don’t rename every table bronze_/silver_ in v1 — noise.
- Don’t dual-write SQLite + DuckDB + Postgres “just in case.”
- Don’t store bronze blobs inside the seed shipped on GitHub (too large). Keep bronze local/CI artifact; ship silver seed.

---

## 5. Crate responsibilities (cleaned)

### 5.1 Botanica

| Responsibility | In scope |
|----------------|----------|
| Schema + migrations | Yes |
| Taxonomy + knowledge CRUD/query | Yes |
| Inventory CRUD (library API) | Yes (Budsy calls it) |
| Ingest CLI (`ingest`, `discover`) | Yes |
| Seed DB build + MANIFEST | Yes |
| UI | No |
| Camera | No |
| Plant ID ML models | No (may store results JSON in L3) |

**Engine (truth today):** DuckDB, not SQLite/SQLx. Docs must match code.

### 5.2 Budsy

| Responsibility | In scope |
|----------------|----------|
| App lifecycle, UI | Yes |
| Open seed / local DB path | Yes |
| L3 writes | Yes |
| CrabCamera integration | Yes |
| External plant ID API orchestration | Yes |
| Own parallel botanica schema reimplementation | **No — delete/park** |
| Enterprise cannabis / MSO vapor docs as product | **No — archive** |

### 5.3 CrabCamera

| Responsibility | In scope |
|----------------|----------|
| Capture frames / recording | Yes |
| Remain independent publishable crate | Yes |
| Know about Botanica taxa | **No** |

Integration rule: Budsy maps capture → `plant_photos.file_path` (+ optional analysis JSON). CrabCamera stays plant-agnostic.

---

## 6. Current state audit (as of 2026-07-08)

### 6.1 Botanica — real

- DuckDB connection wrapper, migrations for 18 tables, indexes
- Family/genus/species types + real insert/get queries
- Ingestion modules: POWO, GBIF, USDA, CSV, normalization, bulk (partial)
- Gate2-related USDA work and scrapers live largely under **budsy** datasets/scraper trees
- Tests for core taxonomy path exist

### 6.2 Botanica — vapor / cruft

| Item | Reality |
|------|---------|
| README “SQLite / SQLx / production-ready / used by institutions” | False or outdated |
| `contextlite` feature | Mock TODOs |
| `conservation` IUCN client | Hardcoded mock |
| `darwin-core` search | Returns empty |
| Features `herbarium`, `germplasm`, `api` | No modules |
| `fts` rebuild | No-op |
| `queries/specimens` | Placeholders |
| Bulk default | 3 hardcoded species + Unknown taxonomy |
| COMPREHENSIVE_SPEC / pro features docs | Ahead of code |

### 6.3 Budsy — real

- Thin Slint desktop demo (~95 lines): memory DB, hardcoded plant, mock camera callback
- Path dependency on botanica; crabcamera dep declared
- Python USDA scraper + gate2 ~99-symbol normalized JSON (useful pilot)
- Expo app with service shells (parallel stack)

### 6.4 Budsy — cruft

- ~1MB markdown plans/missions claiming phases “done”
- README still names `botany-db` / `tauri-camera`
- Agriscrape log spam, multiple unfinished scrapers
- Expo reimplements schema instead of using crate
- Enterprise/cannabis strategy docs not required for core architecture

### 6.5 CrabCamera

- Comparatively mature (v0.8.x, substantial src). **Out of cleanup scope** for Session 0–2 unless integration breaks.

---

## 7. Target architecture decisions — LOCKED (Phase 0 owner Q&A)

All **APPROVED** 2026-07-08.

| ID | Decision | Status |
|----|----------|--------|
| D1 | **Three layers L1/L2/L3 in one schema** | ✅ Approved |
| D2 | **Ship seed = L1+L2 only** (zero L3 rows) | ✅ Approved |
| D3 | **DuckDB is the engine** | ✅ Approved |
| D4 | **Cultivated-only scope** | ✅ Approved |
| D5 | **Provenance required for L2** | ✅ Approved |
| D6 | **One plant row per individual** | ✅ Approved |
| D7 | **External ID table before bulk load** | ✅ Approved |
| D8 | **Kill marketing lies before new features** | ✅ Approved |
| D9 | **Budsy cleanup = park, not feature work** | ✅ Approved |
| D10 | **Bronze paths + silver parquet + DuckDB loader; gold later** | ✅ Approved |
| D11 | **No images in git** | ✅ Approved |
| D12 | **CrabCamera stays decoupled** | ✅ Approved |
| D13 | **Remove ContextLite entirely** | ✅ Approved |
| D14 | **Cultivars in v1 schema + seed** | ✅ Approved |
| D15 | **L3 must be sync-capable** | ✅ Approved |

---

## 8. Schema evolution plan (Botanica)

### 8.1 Keep as-is (with truth in docs)

Existing L1/L2/L3 tables listed in `src/migrations/schemas.rs`.

### 8.2 Add before first serious bulk load

```sql
-- External identifiers (merge key)
CREATE TABLE species_identifiers (
  id VARCHAR PRIMARY KEY,
  species_id VARCHAR NOT NULL,
  source VARCHAR NOT NULL,          -- 'usda' | 'powo' | 'gbif' | 'wfo' | ...
  external_id VARCHAR NOT NULL,
  is_primary INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT current_timestamp,
  UNIQUE (source, external_id),
  FOREIGN KEY (species_id) REFERENCES species(id)
);

-- Species hardening (columns or migration ALTERs)
-- scientific_name VARCHAR
-- taxonomic_status VARCHAR  -- accepted | synonym | unresolved
-- rank VARCHAR              -- species | subspecies | variety | ...
```

```sql
-- Cultivars (phase after species bulk works)
CREATE TABLE cultivars (
  id VARCHAR PRIMARY KEY,
  species_id VARCHAR NOT NULL,
  cultivar_name VARCHAR NOT NULL,
  trade_name VARCHAR,
  source VARCHAR,
  FOREIGN KEY (species_id) REFERENCES species(id)
);
```

```sql
-- plants hardening
-- health_status VARCHAR NOT NULL DEFAULT 'unknown'
-- cultivar_id VARCHAR NULL
```

### 8.3 Quarantine (instead of Unknown taxa)

```sql
CREATE TABLE ingest_quarantine (
  id VARCHAR PRIMARY KEY,
  source VARCHAR NOT NULL,
  external_id VARCHAR,
  raw_name VARCHAR,
  reason VARCHAR NOT NULL,
  payload_hash VARCHAR,
  created_at TIMESTAMP DEFAULT current_timestamp
);
```

### 8.4 Explicitly deferred tables

Herbarium specimens as first-class Darwin Core warehouse, germplasm passport data, GraphQL API — **not** in v1 schema work.

---

## 9. Ingestion architecture

### 9.1 Pipeline stages

```
[Source fetch] → bronze files (optional persist)
       → parse/normalize (controlled vocab)
       → resolve taxon (identifiers / name match)
       → write L1 (if new) + L2 facts + provenance
       → quarantine if unresolved
       → rebuild search aids
       → emit MANIFEST stats
```

### 9.2 Source priority

| Priority | Source | Role | Notes |
|----------|--------|------|-------|
| P0 | Curated master cultivated list | Scope gate | You decide what “counts” |
| P0 | USDA PLANTS bulk/CSV | Traits backbone (NA-heavy) | Gate2 proved path |
| P1 | POWO | Synonyms, distribution, uses | Rate limits + CC BY attribution |
| P1 | GBIF | Vernacular names | Rate limits + attribution |
| P2 | WFO / other name backbones | Resolution | Research spike |
| P2+ | GRIN, crop ontologies, seed catalogs | Cultivars / crops | Per-license |

### 9.3 Rate limits & physical limits (research spike outputs)

The spike must answer, in writing:

| Question | Artifact |
|----------|----------|
| Max polite RPS / daily quota per source | `docs/sources/<source>.md` |
| Auth requirements | same |
| Bulk download vs API-only | same |
| Expected row counts at cultivated scope | MANIFEST estimate |
| Wall-clock for full P0+P1 load | measured sample → extrapolate |
| DuckDB file size at 5k / 20k / 50k taxa | measured |
| GitHub strategy (see §11) | decision record |

### 9.4 Idempotency

- Prefer `INSERT` guarded by unique `(source, external_id)` or provenance hash.
- Re-run safe: same payload → no duplicate facts.
- Schema migrations versioned; seed rebuild is allowed to be “burn and rebuild” until v1.0 freeze.

---

## 10. Packaging & distribution

### 10.1 Repository layout (Botanica target)

```
botanica/
  docs/
    ARCHITECTURE.md          ← this file
    sources/                 ← per-source research (spike)
    RUNBOOK_INGEST.md        ← truth-aligned runbook
  data/
    README.md                ← how seed is built; what is committed
    manifests/
      botanica-cultivated-v0.1.json
    # seed file strategy: see §11
  src/                       ← crate
  scripts/
    build_seed.sh|ps1
    clean_docs.ps1           ← cruft archival helper
```

### 10.2 MANIFEST.json (required fields)

```json
{
  "artifact": "botanica-cultivated-v0.1",
  "built_at": "ISO-8601",
  "engine": "duckdb",
  "schema_version": "0.4.0",
  "counts": {
    "families": 0,
    "genera": 0,
    "species": 0,
    "traits": 0,
    "vernacular_names": 0
  },
  "sources": [
    {"name": "usda_plants", "license": "public domain", "retrieved_at": "...", "record_count": 0}
  ],
  "scope": "cultivated_human_use_v1",
  "l3_rows": 0
}
```

### 10.3 Crate API surface (product-facing)

Keep small and honest:

- `BotanicalDatabase::{memory, file, migrate, ...}`
- queries: family, genus, species, search (real)
- inventory queries for plants/care/photos (implement properly when Budsy needs them)
- `ingestion` behind feature flag
- Optional features only when non-mock

---

## 11. GitHub & artifact storage strategy

### 11.1 Constraints

| Constraint | Guidance |
|------------|----------|
| Soft file limit | ~100MB per file (GitHub warning / push issues above) |
| Repo hygiene | Prefer &lt;1GB total |
| LFS | OK for one seed blob if needed |
| Releases | Preferred for large binaries |

### 11.2 Distribution model (owner decision Q3 — parquet-first)

```
data/
  silver/
    families.parquet
    genera.parquet
    species.parquet
    cultivars.parquet
    traits.parquet
    ...
  manifests/
    botanica-cultivated-vX.Y.json
scripts/
  load_seed   → builds local botanica-cultivated.duckdb from parquet
```

| Artifact | GitHub | Runtime |
|----------|--------|---------|
| Per-table parquet (silver) | Yes (primary) | Source of truth for publish |
| MANIFEST + checksums | Yes | Required |
| Built `.duckdb` | Optional Release/CI artifact | Budsy may ship or build-on-first-run |
| Bronze raw dumps | No (local/CI only) | Reproducibility for maintainers |

Size still measured in Phase 4; if silver set exceeds repo comfort, parquet goes to Release assets with same loader.

### 11.3 Columnar / lakehouse door

Parquet silver **is** the columnar door. DuckDB is the query engine over it. Gold = views/marts later. No separate “make it columnar” project.

---

## 12. Budsy integration contract (park-ready)

### 12.1 Runtime contract

1. On first run: copy seed `botanica-cultivated-vX.duckdb` → user data dir (or open read-write copy).
2. Call `migrate()` for forward schema.
3. All inventory UX reads/writes L3 via Botanica APIs (to be completed).
4. CrabCamera capture → write file under app media dir → insert `plant_photos`.
5. Plant ID: external API → resolve to `species_id` via `species_identifiers` or name search → create/link `plants` row.

### 12.2 What Budsy must stop doing

- Maintaining a second SQLite schema that shadows Botanica
- Claiming published crate names that don’t match
- Treating mission YAML “done” as product truth

### 12.3 Clean park state for Budsy

| Keep | Archive / ignore |
|------|------------------|
| `src/main.rs` (honest demo OK) | Root enterprise/*.md strategy bloat |
| `ui/budsy.slint` | `missions/done` as authority |
| `botanica_usda/` scrape tools (move useful bits to botanica later) | Expo full reimplementation (park branch or `archive/`) |
| Cargo.toml path dep on botanica | Fake “production ready” README claims |
| Link to this architecture | Duplicate architecture essays |

---

## 13. Execution plan (phased, auditable)

Each phase has **entry criteria**, **work**, **exit criteria**. Do not start N+1 until exit of N is met (unless marked parallel).

---

### Phase 0 — Plan audit (THIS SESSION FAMILY)

**Goal:** Architecture sound enough to execute without redesign thrash.

| Step | Work | Exit |
|------|------|------|
| 0.1 | Publish this doc in botanica + mirror in budsy | Files present |
| 0.2 | Human audit: decisions D1–D12 accept/amend | Amended doc committed |
| 0.3 | Create beads epics for Phases 1–5 | `bd list` shows epics |
| 0.4 | Freeze scope: cultivated only; no vapor features | Written in ARCHITECTURE |

**Exit Phase 0:** Owner signs off on §7 decisions (comment or commit).

---

### Phase 1 — Cruft clean & honesty (Botanica + Budsy park)

**Goal:** Repos reflect reality; safe to build on.

#### 1A Botanica (required)

| Step | Work | Exit |
|------|------|------|
| 1A.1 | Rewrite README to DuckDB + honest status | No SQLite/SQLx claims; no “used by institutions” |
| 1A.2 | Align CHANGELOG/ROADMAP with code | Version story consistent |
| 1A.3 | Mark or gate mock modules: contextlite, conservation mocks | Docs say mock OR feature off by default |
| 1A.4 | Archive or demote COMPREHENSIVE_SPEC / pro-features as `docs/archive/` | Not presented as implemented |
| 1A.5 | Fix INGESTION_RUNBOOK for DuckDB | Commands work |
| 1A.6 | `cargo test` green on default features | CI-able |

#### 1B Budsy (park clean — recommended same push window)

| Step | Work | Exit |
|------|------|------|
| 1B.1 | README rewritten: real deps, demo status | Honest |
| 1B.2 | Move strategy/mission bloat to `docs/archive/` | Root readable |
| 1B.3 | Note Expo as experimental/parked | One primary app path declared |
| 1B.4 | Add `docs/BOTANICA_STACK_ARCHITECTURE.md` pointer | Linked |
| 1B.5 | Leave functional demo if it builds; don’t expand features | Parked |

**Exit Phase 1:** New contributor is not lied to; architecture is findable in both repos.

---

### Phase 2 — Schema harden (Botanica only)

**Goal:** Schema can survive bulk load + inventory story.

| Step | Work | Exit |
|------|------|------|
| 2.1 | Add `species_identifiers` | Migration + types |
| 2.2 | Species: `scientific_name`, `taxonomic_status` | Migration + queries updated |
| 2.3 | Plants: `health_status` (+ optional cultivar_id later) | Migration |
| 2.4 | `ingest_quarantine` | Migration |
| 2.5 | Remove/replace bulk “Unknown family” path | Code path gone |
| 2.6 | Inventory query stubs → real CRUD for `plants` | Tests |
| 2.7 | Schema version string real | `check_schema_version` not fake-only |

**Exit Phase 2:** Fresh DB migrates; can insert species with USDA id; can insert 9 plants with health statuses; tests pass.

---

### Phase 3 — Ingest path that doesn’t lie (Botanica)

**Goal:** One repeatable pipeline from source → silver DuckDB.

| Step | Work | Exit |
|------|------|------|
| 3.1 | Master cultivated list format + loader | CSV/JSON schema documented |
| 3.2 | USDA CSV/bulk → traits + requirements + provenance | N≥100 taxa automatic |
| 3.3 | Name resolution policy documented | No Unknown taxa |
| 3.4 | `scripts/build_seed` produces DB + MANIFEST | Reproducible |
| 3.5 | Search by scientific + common name works | Demo query |
| 3.6 | Import gate2 pilot data from budsy datasets | Continuity |

**Exit Phase 3:** `build_seed` creates non-toy DB (≥100 species, real traits, provenance rows).

---

### Phase 4 — Source research spike (Botanica)

**Goal:** Know how to get “all cultivated” and how long/big.

| Step | Work | Exit |
|------|------|------|
| 4.1 | Document USDA full bulk path + counts | `docs/sources/usda.md` |
| 4.2 | Document POWO limits + cultivated filter strategy | `docs/sources/powo.md` |
| 4.3 | Document GBIF vernacular strategy | `docs/sources/gbif.md` |
| 4.4 | Time/size extrapolation table | In ARCHITECTURE appendix or MANIFEST notes |
| 4.5 | Choose GitHub storage strategy per §11 | ADR note in docs |

**Exit Phase 4:** Written plan for full load with numbers; no fantasy.

---

### Phase 5 — Full cultivated load + publish artifact

**Goal:** Ship the straw.

| Step | Work | Exit |
|------|------|------|
| 5.1 | Run full P0 load | Counts in MANIFEST |
| 5.2 | Run P1 sources within legal/rate limits | Attribution complete |
| 5.3 | Quality report (null rates, coverage) | `data/manifests/quality-vX.json` |
| 5.4 | Publish artifact (git / LFS / Release) | Downloadable |
| 5.5 | Tag botanica release | Semver |

**Exit Phase 5:** Offline DB meets agreed coverage bar (set in Phase 0/4; recommended v1 bar below).

#### Coverage bar for v1 seed — ACCEPTED (Phase 0)

| Metric | Target |
|--------|--------|
| Species (cultivated scope) | ≥ 5,000 (stretch 15–20k) |
| Cultivars | Included in v1 load strategy (Q2); coverage metric refined in Phase 4 |
| With ≥1 English vernacular | ≥ 60% |
| With ≥3 core traits | ≥ 40% |
| Provenance rows | ≥ 1 per ingested source record |
| L3 rows in seed | 0 |
| Distribution | Parquet silver + load script (Q3); size measured Phase 4 |

---

### Phase 6 — Budsy vertical slice (after Botanica seed exists)

**Goal:** Prove money path without expanding scope early.

| Step | Work | Exit |
|------|------|------|
| 6.1 | Open seed file on disk (not only memory) | Works |
| 6.2 | Create plant linked to real species | L3 write |
| 6.3 | CrabCamera real capture → plant_photos | Not mock |
| 6.4 | Search knowledge for care traits | Display |
| 6.5 | Optional: PlantNet/other ID → species resolve | Best-effort |

**Exit Phase 6:** Demo you can record: capture → inventory row → show knowledge.

---

## 14. Beads tracking (substrate)

Beads (`bd`) is available on this machine. Recommended:

```text
botanica/   bd init → epics for Phase 1A, 2, 3, 4, 5
budsy/      bd init → epic for Phase 1B + Phase 6 (deferred)
```

Epic titles (suggested):

| Bead | Title |
|------|-------|
| E0 | Architecture audit sign-off |
| E1 | Botanica honesty + cruft clean |
| E1b | Budsy park clean |
| E2 | Schema harden L1/L2/L3 |
| E3 | Non-toy ingest + build_seed |
| E4 | Source research spike |
| E5 | Full cultivated load + publish |
| E6 | Budsy vertical slice |

This markdown remains source of truth; beads are execution queue.

---

## 15. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Source rate limits block “full scrape” | Prefer bulk downloads; cache bronze; multi-day runs |
| License incompatibility | Per-source docs; provenance; drop source if needed |
| Name collisions / synonym hell | External IDs + quarantine; don’t fake hierarchy |
| Scope creep to wild flora | D4 enforced in master list |
| DuckDB + mobile Budsy mismatch | Desktop-first; mobile may use export subset or different embed later |
| Two-hour overconfidence | Phases 0–3 are “session-sized”; 4–5 are data-sized |
| Doc rot again | README must link ARCHITECTURE; archive old specs |

---

## 16. Open questions — RESOLVED (Phase 0 audit)

Recorded from owner Q&A session.

| # | Question | Decision | Notes |
|---|----------|----------|-------|
| Q1 | Primary Budsy UI for next 6 months? | **Slint desktop** | Expo parked until seed + CrabCamera vertical slice works |
| Q2 | Cultivar modeling in v1 seed? | **Full cultivar modeling in v1** | Cultivars are first-class in schema **and** seed load, not deferred |
| Q3 | Seed distribution format? | **Parquet tables + load script** | Columnar from day one; DuckDB built by loader (or ATTACH/read parquet). GitHub stores parquet set + MANIFEST + build script, not only a monolithic blob |
| Q4 | Cannabis-specific product track? | **Separate vertical later** | Not Botanica schema special-case; future Budsy skin/module |
| Q5 | Multi-user / cloud sync L3? | **Sync required for app to be functional** | Local-only is not enough long-term. Architecture must allow L3 sync (device ↔ cloud or multi-device). Implementation can lag schema hooks, but design must not paint into local-only corner |
| Q6 | ContextLite “AI insights”? | **Remove** | ContextLite was owner’s other service — disregard. Strip feature/deps/mocks. Intelligence path = DuckDB/columnar knowledge + app logic, not ContextLite |

### Implications locked by these answers

| Decision | Architecture impact |
|----------|---------------------|
| Q2 full cultivars | Phase 2 includes `cultivars` + identifiers; Phase 3/5 seed must have a cultivar source strategy (not species-only) |
| Q3 parquet-first | §10–§11 prefer `data/silver/*.parquet` (+ optional built `.duckdb` as CI artifact/Release), loader script is product surface |
| Q5 sync-capable L3 | Add early: stable plant UUIDs (already), `updated_at`, optional `user_id`/`device_id` placeholders, no assumptions that DB file is single-device forever. Full sync protocol = later phase, not Phase 1–3 blocker |
| Q6 remove ContextLite | Phase 1A deletes `contextlite` module, Cargo feature/dep, README AI claims |

---

## 17. Success metrics

| Horizon | Metric |
|---------|--------|
| End Phase 1 | Zero false “production-ready institution” claims in active docs |
| End Phase 3 | Seed ≥100 real cultivated species, reproducible build |
| End Phase 5 | v1 coverage bar met; artifact published |
| End Phase 6 | Recorded demo: camera → plant row → knowledge traits |
| Strategic | Botanica is the default offline cultivated knowledge file for Budsy |

---

## 18. Appendix A — Layer ↔ table map (current + planned)

| Layer | Table | Status |
|-------|-------|--------|
| L1 | families, genera, species | Exists |
| L1 | species_identifiers, cultivars | Planned |
| L2 | synonyms, vernacular_names, distribution_regions, traits, seasonal_characteristics, cultivation_requirements, ecological_interactions, uses, media, provenance | Exists |
| L2 | ingest_quarantine | Planned |
| L3 | plants, plant_photos, care_activities, environments, cultivation_records | Exists (thin) |
| Meta | schema_version / migrations history | Weak today — harden |

---

## 19. Appendix B — What we sifted out of raw intent

| Your intent | Architecture response |
|-------------|----------------------|
| World-class cultivated compendium | L1+L2 seed + provenance + finite master list |
| My 9 plants various health | L3 one-row-per-individual + health_status |
| CrabCamera → Budsy → local crops | Capture to plant_photos; knowledge join on species_id |
| Webcam / plant operation later | Same L3; add sites/cameras table later — don’t block |
| Clean both repos, park Budsy | Phase 1A+1B then Botanica-only data phases |
| Research scrape limits | Phase 4 spike docs |
| GitHub size unknown | Measure → §11 decision tree |
| Bronze/silver/gold later | Paths + provenance + parquet export door; no lakehouse now |
| Two-hour setup fantasy | Phases 0–3 session-compressed; full load is Phase 5 |

---

## 20. Appendix C — Audit checklist (use this to sign off Phase 0)

- [x] Q1–Q6 answered (see §16)
- [x] D1–D15 accepted (see §7)
- [x] v1 coverage bar accepted (Phase 5 bar)
- [x] Medallion/parquet model accepted (Q3/D10)
- [x] CrabCamera decoupled (D12)
- [x] Personal data never ships in OSS seed (D2)
- [x] L3 sync-capable (D15) — hooks Phase 2; protocol later
- [x] Doc mirrored / linked from Budsy
- [ ] Cruft lists double-checked during Phase 1 execution

**Sign-off:**

```
Auditor: Michael A. Kuykendall (interactive Q&A)
Date: 2026-07-08
Result: APPROVED
Amendments: none on D1–D15; Q2/Q3/Q5/Q6 overrode initial recs
  (cultivars v1, parquet-first, sync-required, remove ContextLite)
```

### Q&A log

| When | Answers |
|------|---------|
| 2026-07-08 | Q1 Slint · Q2 full cultivars v1 · Q3 parquet+loader · Q4 cannabis vertical later · Q5 sync required · Q6 remove ContextLite |
| 2026-07-08 | Coverage bar: accept recommended |
| 2026-07-08 | D1–D15: all Approve |

---

## Key Decisions (summary)

1. **Botanica = OSS knowledge straw (L1+L2) + inventory schema (L3 empty in seed).**  
2. **Budsy = app + L3 writes + camera; parked clean before features; Slint desktop primary (Q1).**  
3. **CrabCamera stays independent capture.**  
4. **Silver = parquet tables + load script; DuckDB local engine; bronze optional; gold later (Q3).**  
5. **Cultivated finite scope including cultivars in v1 (Q2); external IDs + quarantine; no Unknown taxa.**  
6. **Honesty cleanup before scrape; remove ContextLite (Q6).**  
7. **L3 sync-capable design (Q5); full protocol after seed works.**  
8. **Cannabis = later Budsy vertical, not Botanica core (Q4).**  
9. **Execute in phases 0→6 with hard exit criteria.**

---

## PR Plan (implementation sequencing)

| PR | Repo | Title | Depends on | Notes |
|----|------|-------|------------|-------|
| PR0 | botanica + budsy | docs: stack architecture | — | This document + mirror |
| PR1 | botanica | docs: honesty pass (README/runbook/archive vapor) | PR0 | Phase 1A |
| PR2 | budsy | docs: park clean + architecture pointer | PR0 | Phase 1B |
| PR3 | botanica | feat: schema identifiers, quarantine, plant health | PR1 | Phase 2 |
| PR4 | botanica | feat: real inventory plant CRUD + tests | PR3 | Phase 2 |
| PR5 | botanica | feat: USDA/master-list seed builder + MANIFEST | PR3 | Phase 3 |
| PR6 | botanica | docs: source research spike results | PR5 | Phase 4 |
| PR7 | botanica | data: v1 cultivated seed publish | PR6 | Phase 5 |
| PR8 | budsy | feat: open seed + L3 + CrabCamera capture slice | PR7 | Phase 6 |

---

*End of architecture document. Audit §20 before writing production code beyond Phase 1 honesty fixes.*
