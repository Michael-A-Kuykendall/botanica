# Release process (data + crate)

## Products

| Product | Location | Public? |
|---------|----------|---------|
| **Cultivated KEEP silver** | `data/silver_keep/*.parquet` | **Yes — primary share** |
| KEEP MANIFEST + quality | `data/manifests/botanica-keep-*.json`, `quality-keep-*.json` | Yes |
| Full warehouse silver | `data/silver/*.parquet` | Optional (wider; includes non-KEEP) |
| DuckDB engine file | `data/botanica-cultivated-v*.duckdb` | Optional Release asset; usually gitignored |

## Preflight

1. `python scripts/export_keep_set.py --tag <tag>`  
2. Check `quality-keep-<tag>.json` on **KEEP** denominator  
3. `Get-ChildItem data/silver_keep` — total should stay well under GitHub limits (~20 MB today)  
4. No L3 plants in seed (`plants` = 0)

## Data tag

```bash
git tag data-v0.2.0
git push origin data-v0.2.0
# GitHub Release: attach zip of data/silver_keep + data/manifests/botanica-keep-*.json + quality-keep-*.json
```

Zip recipe (PowerShell):

```powershell
Compress-Archive -Path data/silver_keep, data/manifests/botanica-keep-baseline.json, data/manifests/quality-keep-baseline.json, data/manifests/keep-membership.json -DestinationPath botanica-keep-data-v0.2.0.zip
```

## Crate bump

Only when Rust API/schema changes. See `docs/VERSIONING.md`. Data tags and crate versions are independent.

## License / sources

USDA public domain; POWO/GBIF CC BY 4.0 — keep attribution in MANIFEST `sources` / provenance.
