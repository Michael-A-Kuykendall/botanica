# Data sources

| Source | Role | License | Scraper |
|--------|------|---------|---------|
| [USDA PLANTS](usda.md) | NA taxonomy + hort traits | Public domain | `budsy/botanica_usda` |
| [POWO](powo.md) | Accepted names, synonyms, distribution, lifeform/climate | CC BY 4.0 | `scripts/scrape_powo.py` |
| [GBIF](gbif.md) | Vernacular names only | CC BY 4.0 (dataset varies) | `scripts/scrape_gbif_vernacular.py` |

**Always** fail-fast: `debug` → `smoke` → `sample/full` (see `docs/SCRAPE_FAIL_FAST.md`).

**Field maps:** [usda_field_map.md](usda_field_map.md)

**Quality:** `python scripts/quality_report.py --tag <name>`

**Provenance model:** [../PROVENANCE.md](../PROVENANCE.md)

