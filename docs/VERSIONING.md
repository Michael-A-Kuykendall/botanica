# Versioning strategy (Botanica knowledge + crate)

Two version axes. Do not conflate them.

## 1. Crate / schema semver (`Cargo.toml`)

| Bump | When |
|------|------|
| **MAJOR** | Breaking Rust API or breaking schema without migration path |
| **MINOR** | New tables/columns/features, backward-compatible API |
| **PATCH** | Fixes, docs, non-breaking |

Current: **0.3.x** crate, schema stamped **0.4.0** in `schema_meta`.

When schema stabilizes for external consumers → **1.0.0** crate + schema lock.

## 2. Knowledge seed / silver data versions

Columnar exports are the redistributable product. Version them independently of the crate when needed.

### Artifact naming

```
botanica-cultivated-vMAJOR.MINOR[.PATCH]
  data/manifests/botanica-cultivated-vX.Y.Z.json
  data/silver/*.parquet          # or release zip of silver/
  optional: botanica-cultivated-vX.Y.Z.duckdb  (Release asset, not always in git)
```

| Component | Meaning |
|-----------|---------|
| **MAJOR** | Breaking change to table set or column semantics (consumers must remigrate) |
| **MINOR** | More species / richer traits / new optional columns / new source batches |
| **PATCH** | Corrections, re-exports, provenance fixes without new semantics |

### When to cut a data version

Ship a numbered silver release when **all** of:

1. Schema version is recorded and migrations apply cleanly  
2. MANIFEST has counts + sources + licenses  
3. L3 inventory rows in the artifact = **0**  
4. At least one of:
   - Taxonomy milestone (e.g. full USDA Species list loaded)  
   - Trait coverage milestone (research gates: 100 / 1k / full)  
   - Multi-source merge milestone (POWO / GBIF vernaculars)  

### Suggested milestones (from Budsy research)

| Data tag | Criteria (approx) |
|----------|-------------------|
| **v0.1.0** | Schema 0.4 + USDA Species taxonomy bulk + gate2 trait pilot (current trajectory) |
| **v0.2.0** | Gate 5A.3: ≥1k species with Tier 1 traits trending ≥65% where USDA has data |
| **v0.3.0** | POWO uses sample gate passed + bulk uses for cultivated subset |
| **v0.4.0** | GBIF vernacular pass + search quality |
| **v1.0.0** | Research bar: ~30–50k cultivated focus + 70–80% Tier 1 where sources allow; public docs |

`v0.1.0` may still include full USDA Species checklist (broader than “cultivated-only”); tag notes must say so. Cultivated filter / `is_cultivated` can refine without MAJOR if additive.

### Git vs GitHub Releases

| Artifact | Prefer |
|----------|--------|
| Code, migrations, scripts, docs | git |
| Small silver parquet set (&lt; ~50 MB total) | git OK |
| Large bronze dumps (PlantSearch JSON ~100MB+) | **gitignored**; rebuild script or Release |
| Full `.duckdb` | Release asset or local build; usually gitignore |
| MANIFEST | always with the silver set |

See also `data/README.md` and architecture § packaging.

### MANIFEST fields (required for any release)

- `artifact`, `built_at`, `schema_version`, `engine`  
- `counts` (species, traits, vernacular, plants=0, …)  
- `sources[]` with license + record_count  
- `scope` string (e.g. `usda_species_taxonomy + gate2_traits`)  
- `silver_files[]` or checksum list  

### Consumer contract

1. Prefer **parquet silver + `load_silver_parquet`** over opaque duckdb when integrating.  
2. Pin both **crate minor** and **data tag** in Budsy.  
3. Never ship user L3 rows in a public seed.

---

## Shredding work into versions (practice)

After each gate:

1. Update MANIFEST `artifact` / scope notes  
2. Tag git: `data-v0.1.0` or monorepo tag `botanica-data-v0.1.0`  
3. Optional GitHub Release with silver zip + MANIFEST  
4. Bump crate only if API/schema changed  

Do **not** invent new version schemes mid-scrape; cut tags at gate exits.
