# POWO (Plants of the World Online)

| | |
|--|--|
| Base | `https://powo.science.kew.org/api/2` |
| License | CC BY 4.0 (attribute RBG Kew / POWO) |
| Auth | None; **User-Agent required** (403 without) |
| Official note | Prefer [pykew](https://github.com/RBGKew/pykew) for bulk conscience |

## Endpoints used

| Call | Path | Purpose |
|------|------|---------|
| Search | `GET /search?q={name}` | Resolve scientific name → `fqId` |
| Taxon | `GET /taxon/{fqId}` | Detail payload |

## Fields available (2026 API)

| Field | Maps to |
|-------|---------|
| `name`, `authors`, `taxonomicStatus` | identity / accepted name |
| `fqId` | `species_identifiers` source=`powo` |
| `synonyms[]` | `synonyms` |
| `locations[]` | `distribution_regions` (WGSRPD codes) |
| `lifeform` | `traits.lifeform` |
| `climate` | `traits.climate` |

## Ceiling: uses

**Plant uses are not exposed** on `api/2/taxon` (verified 2026-07-09). Research gate “≥50% uses on 1k sample” is **not measurable** via this API.

**Replacement gate:** ≥50% of sample have useful enrichment = match + (synonyms ∨ locations ∨ lifeform ∨ climate).

## Rate / scale

Polite: ~0.3–0.5s between calls per worker; 2–4 workers. Prefer stratified sample then dense (has-trait) set before full 62k.

## Scraper

```bash
python scripts/scrape_powo.py --phase debug
python scripts/scrape_powo.py --phase smoke --names-file data/bronze/name_lists/sample_1000.txt
python scripts/scrape_powo.py --phase sample --names-file data/bronze/name_lists/sample_1000.txt --sample-count 1000
python scripts/enrich_from_bronze.py --powo data/bronze/powo --export-silver
```
