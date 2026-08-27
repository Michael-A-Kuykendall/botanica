# Iterative fill (agile data updates)

Goal: grow **KEEP** coverage for human-relevant slices (houseplants, crops, ornamentals) without reloading 62k wild bulk every time.

## Loop (one sprint)

```text
1. SCORE     quality report + gap lists
2. TARGET    pick a slice (houseplants, hardiness, uses, …)
3. FAIL-FAST debug → smoke on 5–10 names
4. FILL      scrape/allowlist only those gaps
5. MERGE     IDs first; quarantine junk
6. KEEP      python scripts/export_keep_set.py --tag <sprint>
7. QUALITY   quality-keep-<tag>.json
8. SHIP      commit silver_keep/*.parquet + manifests (optional Release)
```

## See what is there

```bash
# Product counts
duckdb -c "SELECT count(*) FROM read_parquet('data/silver_keep/species/*.parquet');"

# Field coverage on KEEP
python scripts/quality_report.py --tag now   # warehouse DB if present
python scripts/export_keep_set.py --tag now  # refreshes quality-keep-*.json

# Priority gap list (houseplants starter included)
python scripts/gap_report.py
# → data/manifests/gap-houseplants.txt
```

## How to add “all houseplants” (example)

1. Maintain a **priority list** (curated names, not full USDA):  
   `data/lookups/priority_houseplants.txt` (one scientific name per line).
2. `python scripts/gap_report.py --list data/lookups/priority_houseplants.txt`  
   → missing names only.
3. For each missing name (or batch):  
   - resolve USDA symbol / POWO / GRIN if possible  
   - fail-fast scrape traits/vernaculars  
   - insert into warehouse (or bronze → enrich).
4. Re-run `export_keep_set.py` so they appear in **silver_keep** if they meet KEEP rule  
   (payload **or** GRIN/FAOSTAT). If a houseplant has no payload and isn’t GRIN-matched,  
   either add a **priority allowlist source** (`source=priority_houseplant`) to KEEP rule  
   or scrape until payload exists.

## KEEP rule (current)

```text
KEEP = traits OR cultivation_requirements OR uses
    OR species_identifiers.source IN ('grin','faostat')
```

Extend later with: `priority_houseplant`, `nc_state`, etc.

## Cadence

| Cadence | Action |
|---------|--------|
| Every fill | gap_report → fill → export_keep → commit parquets |
| Weekly/monthly | quality-keep tag + optional `data-v0.x` Release |
| Never | Unfiltered re-dump of full wild flora as the product |

## Beads / tracking

Open a bead per slice: e.g. “Fill houseplant gap list v1”, “Hardiness free pass 2”.  
Close only when `gap_report` missing count drops and quality JSON is updated.
