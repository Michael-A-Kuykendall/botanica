# Cultivated-Scope Sources (Negative Sweep Allowlist)

**Goal:** Define the authoritative, internationally-sourced allowlist used to mark a
taxon as *cultivated / human-used* (`is_cultivated_scope`) in the VINYL universe, and
to derive the high-confidence `is_definitive` subset.

The sweep is **inclusive**: a taxon is in scope if it appears in ANY allowlist below,
resolved through the canonical taxonomy (POWO/WCVP + our synonym graph). This replaces
the original GRIN-only scope and broadens coverage toward "all human-cultivated plants
on Earth" rather than USDA-centric crops.

## Status (2026-07-10)

- **Already wired in `scripts/build_curation.py`** as cultivation signals (discovered
  during the spike): `grin` (GRIN-Global germplasm) and `faostat` (FAOSTAT crop
  commodities). These were present before this work.
- **WCUPS (Kew) — DONE.** `scripts/extract_wcups.py` parses the 2020 PDF
  (40,240 entries / 39,668 distinct names); `scripts/ingest_wcups.py` tagged 22,056
  matching warehouse species + inserted 5,688 new species (genus→family resolved from
  the PDF hierarchy). WCUPS now signals 27,738 species.
- **ITPGRFA Annex I (FAO) — DONE.** `scripts/ingest_itpgrfa.py` encodes the treaty crop
  list (food-crop genera + explicit forage species, gene-pool approach) and tags 3,374
  warehouse species with source `itpgrfa`.
- **Impact:** vinyl universe 63,074 → **70,461**; warehouse 107,066 → 112,754 species;
  `wcups`=27,738, `itpgrfa`=3,374.
- **Deferred (low marginal gain vs acquisition cost):** Mansfeld (IPK, Oracle-Apex web
  app, no CSV export — would need session-scraping; 6,100 *crops* overlap heavily with
  WCUPS/GRIN) and Genesys (germplasm API — overlaps GRIN). Revisit only if a fuller
  international union is required; both would primarily re-confirm already-in-scope taxa.
- **POWO full (V4/V5) — DONE via WCVP bulk.** The POWO api
  (`powo.science.kew.org/api/2`) is blocked by a Cloudflare challenge from CI/cloud
  hosts, so V4 used the **WCVP Darwin-Core archive**
  (`sftp.kew.org/pub/data-repositories/WCVP/wcvp_dwca.zip`, 2026-06) — the same Kew
  checklist the API serves, with identical IPNI LSIDs. `scripts/ingest_wcvp.py` mapped
  **65,752** warehouse species (59%) to accepted WCVP taxa by binomial and inserted powo
  identifiers (LSID), synonyms, and TDWG WGSRPD distribution + lifeform/climate. Sig
  growth: definitive **67,227**, cultivated scope **77,489**.

## Tier 1 — Global union of human use (primary allowlist)

| Source | Coverage | Access | Notes |
| --- | --- | --- | --- |
| **Kew World Checklist of Useful Plant Species (WCUPS) 2020** | 40,292 species w/ documented human use (10 use categories: medicines, materials, food, animal food, fuels, environmental, social, poisons, gene sources, invertebrate food) | Dataset @ kew.iro.bl.uk (PDF + data file via KNB doi:10.5063/F1CV4G34). Built from 13 datasets incl. MPNS, PROSEA, Useful Plants of New Guinea. | Single best global allowlist. Backbone = IPNI + WCVP/POWO. Already reconciled taxonomy. |
| **GRIN-Global (USDA NPGS)** | 600k+ accessions; ~100k+ taxa | API/CSV | Already in warehouse (USDA 62,558 + backbone 44,508). Core allowlist for crops + wild relatives. |
| **Mansfeld World DB of Agric. & Hort. Crops (IPK Gatersleben)** | 6,100 crop species (excludes forestry + ornamentals) | Web DB (mansfeld.ipk-gatersleben.de) | High-precision crop list; subset of WCUPS but useful as independent signal. |

## Tier 2 — International / treaty / census lists (broadening)

| Source | Coverage | Access | Notes |
| --- | --- | --- | --- |
| **FAO WCA 2022 Crop List** (World Programme for Census of Agriculture) | Botanical crop names w/ CPC/ICC correspondence | FAO Caliper (SPARQL endpoint + PDF manual) | Global agricultural census; captures regionally important crops. |
| **ITPGRFA Annex I** (Intl Treaty on PGRFA, FAO) | 64 globally important food/forage crops + gene pool (crop wild relatives) | FAO plant-treaty Annex I page | Treaty-level authoritative; complements wild relatives. |
| **Genesys** (global PGRFA gateway) | Germplasm accessions worldwide | API (genesys.org) | Broad cultivar/wild-relative coverage; resolves to taxa. |

## Tier 3 — Taxonomic backbone & enrichment (NOT scope-defining alone)

- **POWO / WCVP** (Kew) — canonical taxonomy + synonym resolution. Already scraped (V4 full run).
- **GBIF** — occurrence/distribution backbone; used for synonym resolution, not scope.

## Sweep rule (re-derivation)

```
is_cultivated_scope = name resolves (via synonym graph) to ANY Tier 1 or Tier 2 allowlist entry
is_definitive       = (count of distinct allowlist signals >= 2)  # high-confidence cultivated
```

This keeps the original negative-filter logic ("scrape everything, drop anything with no
cultivation/use evidence") but fixes the original flaw: the source list behind it was
too thin and US-biased. The union above is international and institutionally authoritative.

## Implementation notes

- Download each allowlist to `data/bronze/allowlist/<source>/`, normalize to
  `(canonical_name, source, signal)` rows.
- Resolve each name through POWO/WCVP synonym graph into the warehouse `species` table.
- Rebuild `data/gold/species_curation.parquet` with the union scope (extend
  `scripts/build_curation.py`): add `allowlist_sources` array + `allowlist_signal_count`.
- DB may be reset at any time (`rm data/botanica-cultivated-v0.1.duckdb` + re-ingest).

## Expected impact

Original vinyl universe (GRIN-only scope) = 63,074. Adding WCUPS (40,292) alone +
Mansfeld + FAO + ITPGRFA + Genesys union should push the cultivated-scope universe
materially higher (target: capture the full ~40k+ useful-species set plus crop wild
relatives). `is_definitive` tightens to the well-attested intersection.
