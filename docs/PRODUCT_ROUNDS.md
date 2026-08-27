# Botanica product rounds (measurement plan)

**Purpose:** Answer “where are we?” with **round numbers + exit metrics**, not agent opinion.  
**Source of research:** Budsy OpenSpec + `docs/EXECUTION_PLAN.md` + `docs/ARCHITECTURE.md` Phase 5 bar.  
**Tracker:** `bd` epics/tasks under labels `round-N`. This doc is the scoreboard.

---

## End product (one sentence)

**Botanica** ships a reproducible L1+L2 cultivated-plant knowledge seed (parquet silver + load path + MANIFEST), filled from every permitted P0/P1 source we can legally rate-limit, with blank fields filled **where sources actually have data**, plus OSS repo hygiene matching sibling projects — then **one coordinated version cut** (data tag + crate when schema/API warrants).

| Layer | Ships in seed | Does not ship |
|-------|---------------|---------------|
| L1 taxonomy | families, genera, species, identifiers | fake “Unknown” taxa |
| L2 knowledge | traits, cultivation_requirements, vernaculars, uses, distribution, media URLs, provenance | user inventory |
| L3 | schema only; **0 rows** in public seed | personal plants / photos |
| Product UX | none (Budsy) | camera, sync protocol, paid UI |

---

## How progress is measured (not opinion)

Every full/source gate ends with a **quality report** checked into:

```text
data/manifests/quality-<data-version>.json
```

### Global v1 seed bar (ARCHITECTURE § Phase 5 — accepted)

| Metric | Pass if |
|--------|---------|
| Species in cultivated *scope* | ≥ 5,000 (stretch 15–20k; research ceiling ~30–50k cultivated) |
| ≥1 English vernacular | ≥ **60%** of species in scope |
| ≥3 core traits (practical Tier1) | ≥ **40%** of species in scope |
| Provenance | ≥ 1 row per ingested source record |
| L3 `plants` in seed | **0** |
| Silver + MANIFEST | present; `build_seed` reproduces counts |

**Practical Tier1 fields** (USDA-realistic, fail-fast / coverage):  
`soil`, `moisture`, `mature_height_cm`, `toxicity`  
(Do **not** block gates on USDA hardiness — usually empty even when scrape is healthy.)

**Formal Tier1** (research stretch for v1.0 data tag): hardiness, sunlight, soil, moisture, size, toxicity — count only when *any* source supplies them.

### Per-source gate pattern (always)

```text
debug (print raw+norm, 2–5 ids) → smoke (5–10 from real list + coverage) → full
```

See `docs/SCRAPE_FAIL_FAST.md`. Non-zero exit = stop.

### Data tags (VERSIONING.md)

| Tag | Exit metric (approx) | Status |
|-----|----------------------|--------|
| **data-v0.1.0** | Schema 0.4 + USDA Species taxonomy + any trait pilot | **met in practice** (not tagged public) |
| **data-v0.2.0** | ≥1k species with practical Tier1; HasChar path proven | **met** (~4480 HasChar, ~95% 3+ on that set) |
| **data-v0.3.0** | POWO uses sample gate + bulk uses for cultivated subset | open |
| **data-v0.4.0** | GBIF vernacular pass hits English ≥60% on scope **or** documents ceiling | open |
| **data-v1.0.0** | Global v1 bar + multi-source MANIFEST + public docs honest | open |

**Crate:** `0.3.x` today. Bump **once** at Round 6 with the data cut (minor if API/schema additive; patch if data-only + docs).

---

## Current baseline (2026-07-09 MANIFEST)

| Count | Value |
|-------|------:|
| species | 62,558 |
| families / genera | 610 / 6,972 |
| traits / cult.req / vernaculars | 21,164 / 25,733 / 7,078 |
| cultivars / plants (L3) | 0 / 0 |
| HasChar rich trait universe | ~4,480 of 62k USDA Species |
| Scope note | USDA Species checklist (broader than pure “cultivated”); pre-public |

**Implication:** Breadth is already past the *minimum* species bar. Gaps are **depth** (traits/uses/vernaculars on non-HasChar taxa), **multi-source**, **cultivated filter**, **cultivars**, **OSS release hygiene**, and **one honest version cut**.

---

## Rounds overview

| Round | Name | Goal | Est. agent sessions | Depends |
|------:|------|------|---------------------|---------|
| **0** | Foundation | Schema + USDA tax + HasChar + fail-fast + honesty | done | — |
| **1** | Scoreboard | Field map lock + quality report + baseline % | 1 short | R0 |
| **2** | POWO | Names/uses path: debug→smoke→1k gate→bulk merge | 2–4 | R1 |
| **3** | GBIF vernaculars | Vernacular fill on accepted set | 1–2 | R1 (// R2 ok) |
| **4** | Merge & fill | WCVP/dedupe, cultivated flag, blank-fill, cultivars strategy | 2–3 | R2, R3 |
| **5** | OSS parity | CI, templates, release process, source docs, README truth | 1–2 | // R1–4 |
| **6** | Version cut **once** | Quality green → tag data + bump crate → Release assets | 1 | R4 metrics, R5 |
| **7** | Budsy vertical | App opens seed; plant row; traits (deferred product) | later | R6 |

**Parallelism:** Round 5 can run beside 2–4. Round 3 can start after R1 without waiting for full POWO bulk. Round 6 does **not** ship until R4 coverage numbers are written to quality JSON (even if some sources max out below research stretch).

---

## Round 0 — Foundation (DONE)

| ID | Work | Exit |
|----|------|------|
| R0.1 | Architecture + Q&A sign-off | ARCHITECTURE approved |
| R0.2 | Honesty + ContextLite out | README truth |
| R0.3 | Schema 0.4 L1/L2/L3 hooks | migrations + tests |
| R0.4 | USDA Species bulk + silver + MANIFEST | ~62k species |
| R0.5 | HasChar full trait scrape + seed enrich | ~4.4k rich taxa |
| R0.6 | Fail-fast regimen (`--phase debug/smoke/full`) | SCRAPE_FAIL_FAST.md |

---

## Round 1 — Scoreboard (measurement before more scraping)

| ID | Work | Exit (measurable) |
|----|------|-------------------|
| R1.1 | Formalize USDA → Botanica field map | `docs/sources/usda_field_map.md` (or under docs/) version-locked |
| R1.2 | Quality report tool/script | Produces `data/manifests/quality-*.json` with: species_n, pct_en_vernacular, pct_3plus_practical_tier1, pct_any_trait, pct_uses, null rates per Tier1 field, by_source counts |
| R1.3 | Baseline report on current silver | Committed quality JSON; numbers match MANIFEST ±1% |
| R1.4 | Scope definition for “%” denominators | Document: `all_species` vs `haschar_subset` vs `cultivated_flag` — use both all + haschar in report |

**Round 1 exit:** Anyone can re-run the report and get the same %, without asking an agent “how’s coverage?”

---

## Round 2 — POWO (fill names / uses / distribution blanks)

| ID | Work | Exit |
|----|------|------|
| R2.1 | Source note: API limits, license, endpoints | `docs/sources/powo.md` |
| R2.2 | Fail-fast **debug** (print everything) | 2–5 taxa raw+norm + keys present |
| R2.3 | Fail-fast **smoke** on real sample list | ≥50% of sample have parseable uses **or** documented “uses empty but identity OK” policy in report |
| R2.4 | **1k uses sample gate** (research Decision 5) | Coverage report; **gate pass if ≥50%** of 1k have ≥1 use category; else stop and reassess source |
| R2.5 | Bulk POWO for cultivated / merge set | Provenance POWO; MANIFEST sources[] entry; silver uses/synonyms/distribution updated |
| R2.6 | Seed rebuild + quality report delta | quality JSON shows uses% / synonym% uplift |

**Round 2 exit:** `data-v0.3.0` criteria met **or** written “POWO ceiling” ADR if API blocks bulk (still ship sample).

---

## Round 3 — GBIF vernaculars

| ID | Work | Exit |
|----|------|------|
| R3.1 | Source note | `docs/sources/gbif.md` (vernaculars only; no occurrence dump) |
| R3.2 | Fail-fast debug/smoke | Plumbing + language tags |
| R3.3 | Bulk vernaculars on accepted scientific set | English vernacular rate reported |
| R3.4 | Quality report | Global bar: **≥60% English vernacular** on defined scope **or** max-from-source documented with actual % |

**Round 3 exit:** `data-v0.4.0` criteria met or ceiling documented with numbers.

---

## Round 4 — Merge, cultivated scope, fill blanks

| ID | Work | Exit |
|----|------|------|
| R4.1 | Name resolution / WCVP or POWO accepted-name policy | `docs/NAME_RESOLUTION.md` updated; no Unknown bulk path |
| R4.2 | `is_cultivated` (or equivalent) + master cultivated list strategy | Flag or list; % metrics can filter |
| R4.3 | Cross-source merge rules (prefer POWO name, USDA traits, …) | Written + applied in seed path |
| R4.4 | Cultivar source strategy (Q2: full modeling in v1) | At least path + pilot **or** explicit “0 cultivars until source X” in MANIFEST notes |
| R4.5 | Blank-fill pass: only where source data exists | Quality: practical Tier1 ≥40% on **cultivated scope** (global bar) |
| R4.6 | Spot-check 100 random species | Checklist file or script output; no critical parse bugs |

**Round 4 exit:** Global v1 seed bar metrics green on cultivated scope **or** MANIFEST `notes` list exact shortfalls with source ceilings (honest ship).

---

## Round 5 — OSS repo parity (sibling project bar)

Compare to **shimmy / crabcamera** errata. Measurable = files exist + CI green.

| ID | Work | Exit |
|----|------|------|
| R5.1 | CI workflow (`cargo test`, optional ingest feature matrix) | `.github/workflows/ci.yml` green on default branch |
| R5.2 | Issue/PR templates | `.github/ISSUE_TEMPLATE/*`, PR template |
| R5.3 | DCO check (if other projects use it) | workflow or documented |
| R5.4 | `RELEASE_PROCESS.md` + data-tag procedure | Points at VERSIONING + quality report |
| R5.5 | Source docs index | `docs/sources/README.md` + usda/powo/gbif |
| R5.6 | README/ROADMAP/CHANGELOG match reality | No “planned seed” if seed exists; counts from MANIFEST |
| R5.7 | Optional: GOVERNANCE.md / DEVELOPERS.md if needed for public | Only if shipping public 0.x |

**Round 5 exit:** New contributor clone → CI → build_seed docs path works without tribal knowledge.

---

## Round 6 — One version cut

| ID | Work | Exit |
|----|------|------|
| R6.1 | Preflight: quality JSON vs bar | Checklist all pass or waivers signed in MANIFEST |
| R6.2 | Tag **data-vX.Y.Z** (first public-ready data tag; likely 0.3+ multi-source or 0.2 USDA-complete) | git tag + MANIFEST artifact name aligned |
| R6.3 | Crate bump **once** (0.3.0 → 0.4.0 or 1.0.0 only if API stable) | Cargo.toml + CHANGELOG |
| R6.4 | GitHub Release: silver zip + MANIFEST (+ optional duckdb) | Downloadable |
| R6.5 | Push branch/main as decided; **no brand splash** until owner says | Technical release only |

**Round 6 exit:** Outsiders can pin `botanica = "x.y"` + `data-vA.B.C` without asking us.

---

## Round 7 — Budsy vertical (after seed exists)

Deferred product path; not required for Botanica OSS cut.

| ID | Exit |
|----|------|
| R7.1 Open silver/duckdb from Budsy | Works offline |
| R7.2 Create L3 plant linked to real species | Row written |
| R7.3 Show care traits from L2 | UI displays Tier1 |
| R7.4 Optional CrabCamera → plant_photos | Not mock |

---

## Agent protocol (standing)

1. `bd ready` / open bead for current round task.  
2. State: **“Round N, step R.N.x — exit is …”** before work.  
3. New feed: fail-fast debug → smoke → full.  
4. End of source work: quality report + MANIFEST + bead close with numbers in notes.  
5. Do **not** cut public version mid-scrape; only Round 6.  
6. Do **not** freestyle sources outside EXECUTION_PLAN without updating this doc + research pointer.

---

## Status log

| Date | Round | Note |
|------|-------|------|
| 2026-07-09 | R0 complete | USDA 62k + HasChar 4480 + fail-fast; beads tree for R1–R7 created |
| 2026-07-09 | R1 next | Scoreboard before POWO bulk |
| 2026-07-09 | **R1–R4 data pass** | Quality script + baseline; POWO dense ~2.6k (uses API ceiling); GBIF vern full missing-EN; enrich+silver. See `quality-post_r2_r3.json` + ceilings. |
| 2026-07-10 | **VINYL epic `bot-dh8` opened** | Goal: definitive global cultivated seed (data-v1.0.0). G (gold curation mart) done; V1/V2/V3 done. |
| 2026-07-10 | V1 GOLD | `scripts/build_curation.py` + `docs/GOLD_CURATION.md`; KEEP now driven by `is_definitive`. Parity 18552. |
| 2026-07-10 | V2 BACKBONE | Inserted 44,508 GRIN taxa not in USDA → warehouse 62,558→107,066. `scripts/ingest_grin_backbone.py`. |
| 2026-07-10 | V3 RE-SCORE | Added `is_cultivated_scope`. **Vinyl universe = 63,074** (GRIN-driven); definitive subset 18,552. KEEP re-exported for both gates. |
| 2026-07-10 | V4–V10 next | Depth remains shallow on new 44k (EN vern 26.9%, 3+Tier1 3.3%). POWO-full (V4), uses (V6), media (V8), depth (V9), release (V10) pending. |
| 2026-08-27 | **V4 POWO-full + V5 merge … DONE via WCVP bulk** | POWO api Cloudflare-blocked from host; pivoted to WCVP Darwin-Core archive (same Kew checklist, IPNI LSIDs). `ingest_wcvp.py`: 65,752 species matched (59%); powo ids 2.6k→65.9k; synonyms +342k; distribution +688k (TDWG). Re-scored: definitive 32,768→**67,227**; cultivated scope 70,461→**77,489**. Remaining open: V6 uses, V7, V8 media, V9 depth, V10 quality/cut. |
