# GBIF vernaculars

| | |
|--|--|
| Base | `https://api.gbif.org/v1` |
| Scope | **Vernacular names only** — no occurrence bulk |
| License | CC BY 4.0-class; attribute GBIF + dataset |

## Endpoints

| Call | Path |
|------|------|
| Match | `GET /species/match?name={scientific}` |
| Vernaculars | `GET /species/{usageKey}/vernacularNames` (paginated) |

## Maps to

| Field | Botanica |
|-------|----------|
| `usageKey` | `species_identifiers` source=`gbif` |
| `vernacularName` + `language` | `vernacular_names` source=`GBIF` |

## Rate

Default scraper: 8 workers, 0.05s sleep per request. Adjust if 429.

## Scraper

```bash
python scripts/scrape_gbif_vernacular.py --phase debug
python scripts/scrape_gbif_vernacular.py --phase smoke --names-file data/bronze/name_lists/missing_en_vernacular.txt
python scripts/scrape_gbif_vernacular.py --phase full --names-file data/bronze/name_lists/missing_en_vernacular.txt
python scripts/enrich_from_bronze.py --gbif data/bronze/gbif_vern --export-silver
```

## Coverage bar

Global v1: ≥60% species with ≥1 English vernacular on defined scope.  
If source ceiling is lower, document actual % in quality JSON.
