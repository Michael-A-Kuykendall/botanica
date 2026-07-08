# Botanica execution plan (from Budsy research)

**Source of truth (research):**  
- `budsy/openspec/changes/add-horticultural-database/design.md`  
- `budsy/docs/archive/BOTANICA_FOCUSED_PLAN.md`  
- `budsy/openspec/changes/add-botanica-horticultural-schema/` (ingestion MVP + source mapping)  
- `budsy/botanica_usda/` (USDA gates + API notes)  
- `budsy/openspec/changes/add-horticultural-database/tasks.md`  

**This file:** executable checklist in the botanica repo. Do not freestyle sources outside this plan without updating research.

---

## Product target (research decision)

| Item | Value |
|------|--------|
| Scope | Cultivated / human-used plants first |
| Count | **30–50k** taxa (est. ~35k); not full wild flora |
| Depth | **70–80% Tier 1** hort fields where sources have them |
| Size | ~30–50 MB core |
| Images | URLs + license only |
| Occurrences | Out of v1 |
| Funnel | Free Botanica → paid Budsy |

---

## Source stack (research priority)

| Priority | Source | Job |
|----------|--------|-----|
| P0 | **USDA PLANTS** | Hort traits (habit, tolerances, height, toxicity, wetland, native…) |
| P0 | **POWO / WCVP** | Accepted names, synonyms, distribution, uses (sample 1k uses first) |
| P1 | **GBIF** | Vernacular names only (not occurrence bulk) |
| P2 | **RHS** | If licensed; do not block |
| P2 | **NC State Plant Toolbox** | Regional supplement |
| Out | Reddit/forums | License/quality |

**Ingest order (OpenSpec):** POWO foundation → GBIF vernaculars → USDA traits → search  
**Risk MVP (FOCUSED_PLAN):** USDA-only launch if POWO/RHS slip.

---

## USDA gates (from research tasks 5A)

| Gate | Target | Status |
|------|--------|--------|
| **5A.1** Smoke 10 | ≥70% with ≥3 Tier 1 | ✅ Done (2025-10-26) |
| **5A.2** Pilot 100 | Tier 1 coverage + attribution | ✅ Gate2 pilot data in `budsy/datasets/botanica_usda` + seed enrich |
| **5A.3** 1k batch | Coverage trending ≥65% hardiness/sun/moisture | ⚠️ Ran 2026-07-08: stratified 1k scrape completed; **Tier1 formal accept failed** (0% hardiness/sun/soil/moisture — many taxa lack characteristics; USDA rarely has hardiness). Got plant_type/duration/vernaculars for many. **Redo later** with `HasCharacteristics` filter. |
| **5A.4** Field mapping lock | USDA → Botanica | ⬜ Formalize from mapping docs |
| **Full USDA list** | ~30k+ NA taxa taxonomy + traits | 🔄 Taxonomy bulk from PlantSearch catalog (this sprint) |

---

## Current sprint (faithful to research)

### Done this session family

- [x] Schema L1/L2/L3 + identifiers, cultivars, quarantine, plant health (Phase 2)
- [x] Gate2 trait pilot loaded into seed path
- [x] Honesty cleanup; ContextLite removed
- [x] Architecture doc + Budsy park
- [x] USDA PlantSearch catalog bronze (`data/bronze/usda_catalog/plant_search_pct.json`)
- [x] Genus→family map from PlantProfile Ancestors (`data/lookups/genus_family_usda.csv`, 6972 genera)
- [x] Master species CSV + bulk taxonomy load into silver seed (**62,349** Species-rank USDA taxa)
- [x] MANIFEST with full USDA species counts + gate2 trait enrich (~97 pilot)
- [ ] Gate 5A.3 trait scrape for 1k symbols (uses `botanica_usda` scraper, not freestyle)

### Explicitly not this sprint

- Full 62k trait scrape in one shot (research: batch gates)
- POWO bulk before 1k uses sample
- GBIF occurrence dump
- RHS scrape without partnership

---

## How to run (USDA taxonomy bulk)

```powershell
# 1) Master list (Species rank) from catalog + genus map
python scripts/build_usda_master.py

# 2) Build seed (taxonomy + gate2 trait enrich where symbols overlap)
cargo run --release --bin build_seed -- usda

# 3) Artifacts
#    data/botanica-cultivated-v0.1.duckdb
#    data/silver/*.parquet
#    data/manifests/botanica-cultivated-v0.1.json
```

## Next research-aligned steps (after this sprint)

1. **5A.3** — Sample 1k USDA symbols (stratified), run `botanica_usda` scraper, import traits.  
2. **POWO 1k uses sample** — gate ≥50% before bulk.  
3. **GBIF vernaculars** on accepted species set.  
4. **WCVP dedupe** for cultivated global set (research phase 3–4 taxonomy).  
5. Raise trait coverage toward 70–80% Tier 1.

---

## Status log

| Date | Note |
|------|------|
| 2026-07-08 | Plan doc created from Budsy research; USDA taxonomy bulk execution follows. |
| 2026-07-08 | USDA PlantSearch Species bulk: **62,349** species, 540 families, 6,972 genera. Gate2 enrich: 351 traits, 102 vernaculars. L3=0. Next: 5A.3 trait scrape 1k. |
| 2026-07-08 | Gate 5A.3 scrape 1k stratified symbols via `botanica_usda`. Formal Tier1 accept **failed** (see coverage report). Seed rebuild: ~62.4k species, ~1.8k traits, ~956 vernaculars. Data tag candidate: **data-v0.1.0**. VERSIONING.md added. |
