# Cultivated-source access requirements (for the human)

Ranked by "how much pain in the ass is it for a person to get access" — easiest first.
Precise requirement for each so you know exactly what to do.

## 1. USDA PLANTS Database — EASIEST (no membership, no account)
- **What it is:** US federal checklist; includes a curated **Cultivated Status** flag
  (C = cultivated only, N = native/not cultivated) on each taxon.
- **Access path:** Published as a **GBIF checklist dataset**
  - DOI: `10.15468/t40oqu` · dataset key: `705922f7-5ba5-49ab-a75d-722e3090e690`
  - License: **CC-BY 4.0**
  - 93,969 name usages
- **Exact human action (one-time, ~2 min):**
  1. Create a free GBIF account → https://www.gbif.org/user/register
  2. Set env vars for the script: `GBIF_USER` and `GBIF_PASSWORD`
  3. Run: `python scripts/ingest_usda_cultivated.py --download --apply`
     (script requests a GBIF download job for the dataset and fetches the DwC-A)
- **Effort: minimal** — just the GBIF account + creds. Everything else is automated.
- **Caveat:** US-centric (North America). Still the cleanest single cultivated flag.

## 2. iNaturalist cultivated observations — EASY (no membership)
- **What it is:** 18.2M observations flagged "captive/cultivated."
- **Access:** Free open API (no key for read), CC-BY; GBIF export also available.
- **What you must do (human):** NOTHING (an iNaturalist account is optional).
- **Effort: minimal** to fetch.
- **Caveat (why we deprioritized):** the "captive" flag is noisy — it includes weeds
  growing near humans (verified: Lythrum salicaria = 1,511 "cultivated"). Cleanup cost >
  value. **Not recommended** as a primary signal.

## 3. Missouri Botanical Garden Plant Finder — MODERATE (no membership, but not bulk)
- **What it is:** ~100k+ ornamental plants, botanical-garden curated.
- **Access:** Public web search, no account. **No public bulk/API** — only per-plant pages
  or filtered search.
- **What you must do (human):** None for browsing; but to get it as data you'd have to
  scrape/search systematically (and check reuse terms). No key, no membership.
- **Effort: moderate** (scrape build), license = check.

## 4. Garden.org Plants Database — MODERATE (no membership, license unclear)
- **What it is:** 808,690 community plants w/ care data (zones).
- **Access:** Public web, no account for browsing. **No public bulk/API.**
- **What you must do (human):** None to view; bulk = scrape + license due-diligence.
- **Effort: moderate-high**; license unclear.

## 5. RHS Plant Finder — HARDEST (proprietary + SPA)
- **What it is:** 305,000+ plants, UK horticulture authority. **Best** data.
- **Access:** It's a JavaScript SPA (`PlantsSPAV2`) — data loads from an internal API.
  **No public bulk, no documented API, no data license.**
- **What you must do (human):**
  - **Membership** alone does NOT give bulk data.
  - For research access, contact RHS Science / the horticultural taxonomy team and
    request a data license / collaboration (they do partner with researchers).
  - OR reverse-engineer the internal API (technically possible but violates ToS risk).
- **Effort: high** — requires a human email/negotiation with RHS, and possibly a
  licensing agreement. This is the one that genuinely needs you (a person) to engage.

## Summary decision
- **Do now (no human effort):** USDA PLANTS Cultivated Status via GBIF (CC-BY, clean).
- **Defer / only if needed:** RHS (needs you to contact them), Garden.org (license),
  MOBOT (scrape).
- **Skip:** iNaturalist (noisy).
