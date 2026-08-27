# Provenance

**Provenance** = how we know where each fact came from: source system, license, and (where recorded) import batch / external id.

Botanica is not a single opaque dump. Knowledge rows carry a **source** label; many species also have rows in **`provenance`** and external keys in **`species_identifiers`**.

---

## How provenance is stored

| Mechanism | Table / field | Purpose |
|-----------|---------------|---------|
| Field-level source | `traits.source`, `cultivation_requirements.source`, `vernacular_names.source`, `synonyms.source`, `distribution_regions.source`, `uses.source` | Which pipeline wrote this fact |
| External IDs | `species_identifiers` (`source` + `external_id`) | Stable merge keys (usda, grin, powo, gbif, wikidata, faostat, …) |
| Import event | `provenance` | Species-level “we ingested from X” with license / hash when available |
| Batch log | `data/manifests/*.json` | Human-readable run log: counts, paths, licenses |

**L3 inventory** (user plants) is never in the public seed — no personal provenance there.

---

## Upstream sources (KEEP product)

| Source | License | Role | Typical `source` / id label |
|--------|---------|------|-------------------------------|
| **USDA PLANTS** | Public domain | Taxonomy catalog; Characteristics scrapes (soil, moisture, height, toxicity, …) | `USDA` / `USDA_PLANTS_*`; id `usda` |
| **GRIN taxonomy** (GBIF-hosted zip) | USDA/GRIN — free with attribution | Cultivated / germplasm membership allowlist | id `grin` |
| **FAOSTAT** crop items | Free with attribution | Commercial crop signal (EN name match) | id `faostat` |
| **POWO** (Kew) | CC BY 4.0 | Synonyms, WGSRPD locations, lifeform/climate | `POWO`; id `powo` |
| **GBIF** | CC BY 4.0 | Vernacular names | `GBIF`; id `gbif` |
| **Wikidata** | CC0 | Sparse hardiness | `WIKIDATA`; id `wikidata` |

### Approximate KEEP footprint (data-v0.2 era)

| Layer | Dominant provenance |
|-------|---------------------|
| Species membership | USDA catalog + GRIN (± FAOSTAT) |
| Care traits / cult.req | Mostly **USDA**; some **Wikidata** hardiness |
| Vernaculars | Mostly **GBIF**; some **USDA** |
| Synonyms | **POWO** |
| Distribution | Mostly **POWO**; some **USDA** |
| Uses | *None yet* |

Exact counts: query `data/silver_keep/provenance/*.parquet` and `source` columns, or re-run scorecards after a fill.

---

## KEEP rule vs provenance

A species is in **`silver_keep/`** if it is **`is_definitive`** in the gold curation mart
(`docs/GOLD_CURATION.md`) — i.e. ≥2 independent cultivation/use signals. The mart is the
definitive gate; `export_keep_set.py` reads it when present (`--no-curation` for legacy).

That is a **membership filter**, not a substitute for field-level provenance. A GRIN-only species may have little care data but is still “in” because of the GRIN id.

---

## Consumer obligations (summary)

| Source | What to do |
|--------|------------|
| USDA PLANTS | Public domain — attribution still good practice |
| POWO / GBIF | **CC BY 4.0** — credit in products that redistribute those facts |
| Wikidata | CC0 |
| GRIN / FAOSTAT | Follow their attribution norms; see allowlist JSON under `data/manifests/` |

Product-facing apps should not strip `source` columns when displaying care data.

---

## Manifest files

| File | Contents |
|------|----------|
| `data/manifests/botanica-cultivated-v0.1.json` | Warehouse build: sources[], licenses, counts |
| `data/manifests/botanica-keep-v0.2.json` | KEEP product pointer + counts |
| `data/manifests/grin-allowlist.json` | GRIN run URL + match counts |
| `data/manifests/faostat-allowlist.json` | FAOSTAT run + match counts |
| `data/manifests/wikidata-hardiness.json` | Wikidata hardiness pass stats |
| `data/manifests/quality-keep-*.json` | Coverage metrics (not a source list) |

---

## Query examples

```bash
# Who contributed provenance rows in KEEP?
duckdb -c "SELECT source, count(*) FROM read_parquet('data/silver_keep/provenance/*.parquet') GROUP BY 1 ORDER BY 2 DESC;"

# Where did care requirements come from?
duckdb -c "SELECT source, requirement_type, count(*) FROM read_parquet('data/silver_keep/cultivation_requirements/*.parquet') GROUP BY 1, 2 ORDER BY 3 DESC LIMIT 20;"

# External IDs on a species
duckdb -c "SELECT i.source, i.external_id FROM read_parquet('data/silver_keep/species/*.parquet') s JOIN read_parquet('data/silver_keep/species_identifiers/*.parquet') i ON i.species_id = s.id WHERE s.scientific_name = 'Aloe vera';"
```

---

## Gaps (honest)

- GRIN/FAOSTAT are strong on **identifiers**; not every allowlist hit has a matching rich **`provenance`** row yet.
- Warehouse MANIFEST lists every USDA bronze gate; KEEP is a filtered view of that warehouse.
- **Uses** have no provenance until a uses source is loaded.
- Re-exports should keep `source` columns intact so provenance is not lost.

---

## Related

- [`docs/sources/README.md`](sources/README.md) — per-source operational notes  
- [`data/README.md`](../data/README.md) — artifact layout  
- [`docs/RELEASE_PROCESS.md`](RELEASE_PROCESS.md) — shipping KEEP + manifests  
