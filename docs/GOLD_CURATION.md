# Gold curation layer (the "definitive" gate)

The public product (`data/silver_keep/`) is a **cultivated / human-used** subset of the
warehouse, decided by evidence — not by a single missing trait.

## Why

Earlier KEEP was a negative filter (`traits OR cult.req OR uses OR grin|faostat`). That
was a decent *definitiveness proxy* but opaque: it dropped 44k taxa and could only ever be
a subset of the USDA warehouse. The vinyl goal ("all human-cultivated species, definitive,
not larded with noise") needs a **transparent, multi-signal** inclusion rule.

## Rule

A species is `is_definitive` when it has **≥ N independent cultivation/use signals**
(default `N = 2`). Each signal comes from a *different source family*, so 2 signals =
corroboration across sources (stronger than one GRIN name-match).

| Signal | Source family | Table / field |
|--------|--------------|---------------|
| `grin` | GRIN germplasm (global cultivated checklist) | `species_identifiers.source='grin'` |
| `faostat` | FAOSTAT crop commodity | `species_identifiers.source='faostat'` |
| `wiki` | Wikidata taxon (notability / use statements) | `species_identifiers.source='wikidata'` |
| `powo` | POWO accepted name | `species_identifiers.source='powo'` |
| `trait` | USDA HasChar trait | `traits` row |
| `cultivation` | cultivation requirement | `cultivation_requirements` row |
| `en_vernacular` | English common name | `vernacular_names` (en/eng/en-us/en-gb) |

## Build

```bash
python scripts/build_curation.py --threshold 2 --tag <tag>
# → data/gold/species_curation.parquet
# → data/manifests/curation-<tag>.json   (signal histogram + composition)
```

`export_keep_set.py` now drives KEEP membership from `species_curation.is_definitive`
when the mart exists (use `--no-curation` to fall back to the legacy rule).

## Current ceiling (this mart, existing signals)

| Threshold | Definitive species |
|-----------|-------------------:|
| ≥ 1 signal | 35,837 (57.3%) |
| ≥ 2 signals | 18,552 (29.7%) |
| ≥ 3 signals | 16,799 (26.9%) |

## POWO/WCVP full (2026-08)

The POWO **api** (`powo.science.kew.org/api/2`) is Cloudflare-challenged from CI/hosts,
so the full run was replaced with the equivalent **WCVP bulk Darwin-Core archive**
(`sftp.kew.org/pub/data-repositories/WCVP/wcvp_dwca.zip`, 2026-06) — the same Kew
checklist behind the API, same IPNI LSIDs. `scripts/ingest_wcvp.py` maps warehouse
species by binomial and inserts:

- `species_identifiers` source=**powo** (LSID) — 2,607 → **65,873**
- `synonyms` source=**powo** — 341,984 rows for 49.7k species
- `distribution_regions` source=**powo** (TDWG WGSRPD) — 688,508 rows for 65.7k species
- lifeform/climate extracted from WCVP `dynamicproperties` (reported in manifest)

Re-scored (V5): definitive **32,768 → 67,227**; cultivated scope **70,461 → 77,489**.
`powo` supplies 64,143 signals in the definitive set.

See `docs/PRODUCT_ROUNDS.md` (Round 8 / VINYL epic `bot-dh8`).
