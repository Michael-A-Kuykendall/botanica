# Scrape fail-fast regimen

**Rule:** Never start a multi-hour feed until a tiny run proves plumbing + useful fields.

## Phases (always in order)

| Phase | Command idea | Pass criteria | Time |
|-------|----------------|---------------|------|
| **1. debug** | Print raw + normalized for 2–5 known symbols | Identity + characteristics keys present; ≥2 practical Tier1 fields on ≥1 record | ~30–60s |
| **2. smoke** | 5–10 symbols from the **same list** you will full-run | Plumbing OK + **≥70%** records with ≥3 of {soil, moisture, mature_height_cm, toxicity} | ~1–3 min |
| **3. full** | Entire symbol list | Pre-flight smoke embedded; then run | hours if large |

Practical Tier1 (USDA-realistic): `soil`, `moisture`, `mature_height_cm`, `toxicity`.  
Formal Tier1 also includes hardiness/sunlight — often empty even when scrape is healthy. **Do not block full runs on hardiness.**

## USDA CLI (`budsy/botanica_usda`)

```bash
cd budsy

# 1) Debug — print everything
python -m botanica_usda.cli --phase debug --output ../botanica/data/bronze/ff_debug

# 2) Smoke — first N of your real list + coverage gate
python -m botanica_usda.cli --phase smoke --mode symbols \
  --symbols-file ../botanica/data/bronze/usda_catalog/lists/haschar_all.txt \
  --smoke-count 5 --min-pct-3plus 70 \
  --output ../botanica/data/bronze/ff_smoke

# 3) Full — auto pre-flight then full list
python -m botanica_usda.cli --phase full --mode symbols \
  --symbols-file ../botanica/data/bronze/usda_catalog/lists/haschar_all.txt \
  --smoke-count 5 --min-pct-3plus 70 \
  --output ../botanica/data/bronze/haschar_full \
  --rate 0.55 --concurrency 8
```

Exit non-zero on fail-fast = **do not continue**.

`--skip-fail-fast` exists for emergencies only.

## Batch script (`botanica/scripts/scrape_haschar_batch.py`)

Prefers the same order: use CLI fail-fast first, then batch. Or:

```bash
# Still run CLI smoke before invoking batch
python -m botanica_usda.cli --phase smoke --mode symbols --symbols-file ... 
python scripts/scrape_haschar_batch.py --symbols-file ...
```

## After scrape: seed

```bash
cargo run --release --bin build_seed -- usda
# Check MANIFEST counts; optional small DuckDB query for traits coverage
```

## Agent / human checklist

- [ ] `--phase debug` green  
- [ ] `--phase smoke` green on **production list**, not only DEFAULT_SMOKE_SYMBOLS  
- [ ] Coverage report looks sane  
- [ ] Only then `--phase full`  
- [ ] Rebuild silver; glance MANIFEST  

If smoke fails after a green debug: **list composition** problem (lichens with no chars), not necessarily plumbing.
