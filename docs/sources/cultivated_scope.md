# Cultivated scope (R4)

## Problem

USDA Species checklist is **broader** than “intentionally cultivated” (~62k includes lichens, weeds, wild NA flora). Research target is ~30–50k cultivated.

## v0 measurement approach (no schema break)

Until `is_cultivated` column ships:

| Denominator | Definition |
|-------------|------------|
| `all_species` | All species rows |
| `has_any_trait` | Species with ≥1 `traits` or `cultivation_requirements` row (USDA HasChar / enrich dense set) |
| `quality report` | Reports both; v1 bar checked on `all_species` **and** annotated for `has_any_trait` |

## Future (schema additive)

- `species.is_cultivated BOOLEAN` or `species_identifiers` source=`cultivated_list`
- Master list: POWO uses (when available) ∪ USDA cultivated flags ∪ NC State / RHS if licensed

## Cultivars

Schema ready; **0 rows** until a cultivar source is wired. MANIFEST notes: `cultivars=0 until source`.

## Merge preference

| Concern | Prefer |
|---------|--------|
| Accepted scientific name | POWO when matched |
| Hort traits (soil/moisture/height/tox) | USDA |
| Vernacular multi-lang | GBIF (+ USDA en) |
| Distribution codes | POWO locations + USDA native/introduced |
