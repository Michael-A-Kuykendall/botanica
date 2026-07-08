# Botanica roadmap

Aligned with [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md). This is not a marketing checklist.

## Done

- [x] Taxonomic hierarchy types + CRUD (family / genus / species)
- [x] DuckDB migrations: L1 + L2 reference tables + L3 inventory schema (empty in seed)
- [x] Optional ingest scaffolding (POWO / GBIF / USDA)
- [x] Architecture decisions Phase 0 (Q1–Q6, D1–D15)
- [x] Honesty pass: README, remove ContextLite, archive vapor specs
- [x] Phase 1B Budsy park (sibling repo)
- [x] Phase 2: schema 0.4.0 — identifiers, cultivars, quarantine, plant health, sync hooks, plant CRUD, no Unknown bulk
- [x] Phase 3: gate2 seed + USDA Species bulk taxonomy (62k) + silver parquet + MANIFEST
- [x] Execution plan aligned to Budsy research (`docs/EXECUTION_PLAN.md`)

## In progress / next

- [ ] USDA Gate 5A.3 — 1k symbol trait scrape via `botanica_usda`
- [ ] POWO 1k “uses” sample gate (research Decision 5)
- [ ] GBIF vernacular pass
- [ ] Phase 4 notes: source limits / size (partially superseded by bulk USDA taxonomy)
- [ ] Phase 5: v1 cultivated load (coverage bar in architecture)
- [ ] Phase 6: Budsy vertical slice on real seed

## Explicitly not promised

- Full wild flora of Earth
- Real IUCN production client (mock if enabled)
- Darwin Core as complete herbarium stack
- ContextLite / bundled “AI insights”
