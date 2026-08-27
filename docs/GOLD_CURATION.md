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

To grow the definitive set toward "all cultivated Earth" (~30–50k), add signals:
- **Backbone:** insert the ~45k GRIN taxa not yet in the warehouse (V2).
- **POWO-full:** only 2.6k POWO ids exist today; a full run adds accepted-name +
  distribution + cultivated signal to ~60k taxa (V4).
- **Uses / media:** currently 0 rows (V6, V8).

See `docs/PRODUCT_ROUNDS.md` (Round 8 / VINYL epic `bot-dh8`).
