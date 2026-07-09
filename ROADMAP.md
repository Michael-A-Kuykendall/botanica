# Botanica roadmap

Aligned with [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md). This is not a marketing checklist.

**Progress ruler:** [`docs/PRODUCT_ROUNDS.md`](docs/PRODUCT_ROUNDS.md) · **queue:** `bd ready`

## Done (Round 0)

- [x] Taxonomic hierarchy types + CRUD (family / genus / species)
- [x] DuckDB migrations: L1 + L2 reference tables + L3 inventory schema (empty in seed)
- [x] Optional ingest scaffolding (POWO / GBIF / USDA)
- [x] Architecture decisions Phase 0 (Q1–Q6, D1–D15)
- [x] Honesty pass: README, remove ContextLite, archive vapor specs
- [x] Phase 1B Budsy park (sibling repo)
- [x] Phase 2: schema 0.4.0 — identifiers, cultivars, quarantine, plant health, sync hooks, plant CRUD, no Unknown bulk
- [x] Phase 3: USDA Species bulk (~62k) + HasChar traits (~4480) + silver parquet + MANIFEST
- [x] Fail-fast scrape regimen (debug → smoke → full)
- [x] Execution plan + product rounds + beads R1–R7

## In progress / next (by round)

| Round | Focus | Status |
|------:|-------|--------|
| **R1** | Scoreboard: field map + quality report + baseline % | **next** |
| **R2** | POWO fail-fast → 1k uses gate → bulk | queued |
| **R3** | GBIF vernaculars (// after R1) | queued |
| **R4** | Merge, cultivated flag, fill blanks to v1 bar | queued |
| **R5** | OSS parity (CI, templates, RELEASE_PROCESS) | **parallel** |
| **R6** | **One** data tag + crate bump + Release | after R4+R5 |
| **R7** | Budsy vertical on real seed | deferred |

## Explicitly not promised

- Full wild flora of Earth
- Real IUCN production client (mock if enabled)
- Darwin Core as complete herbarium stack
- ContextLite / bundled “AI insights”
