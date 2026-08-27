# Cultivation-source research map (data-science target: "how many human-cultivated species exist")

Goal: determine, to the best educated truth, the number of species humans actually
cultivate / maintain — by enumerating EVERY credible source of "this taxon is grown or
used by people," then unioning and corroborating via `data/gold/cultivation_signal`.

## Status: discovery  2026-08-27

### Already ingested (in `species_identifiers`)
| Source | Species | What it certifies | Strength |
|--------|--------:|-------------------|----------|
| GRIN-Global (USDA NPGS) | 60,959 | germplasm accessions held/curated | strong cultivation |
| WCUPS (Kew, 2020) | 27,670 | documented human use (10 cats) | strong cultivation/use |
| ITPGRFA Annex I (FAO) | 3,374 | treaty food/forage crops | strong crop |
| FAOSTAT (FAO) | 487 | crop commodities | strong crop |
| USDA PLANTS | 61,900 | NA flora taxonomy (includes natives) | WEAK for cultivation |
| POWO/WCVP (Kew) | 65,873 | global taxonomy of ALL plants | WEAK (taxonomy) |
| GBIF | 57,277 | occurrence records | WEAK (taxonomy/occurrence) |
| Wikidata | 18,253 | crossref existence | WEAK |

Honed cross-check (distinct sources, not kinds):
  union ≥1 cultivation source = 68,709
  ≥2 independent sources      = 22,282  (the corroborated cared-for set)
  ≥2 sources AND maintenance  = 1,881   (immediately serviceable care core)

### Discovered, NOT yet ingested (research-gap to close)
| Source | Reach | What it certifies | Notes / access |
|--------|-------|-------------------|----------------|
| **RHS Plant Finder** | 381,791 results | UK horticultural cultivation authority | Filterable page; hardiness H1-H7, plant types, AGM. Not yet bulk accessible publicly |
| **Garden.org Plants DB** | 808,690 plants | community horticultural care (zones) | Collaborative; includes cultivars |
| **Plant Database USDA hardiness maps** | — | USDA zones | merged already? |
| **Missouri Botanical Garden / Plant Finder** | ~100k? | ornamental cultivation | mobot.org plantfinder |
| **Crocus, RHS trials** | — | garden cultivars | IDs overlap |
| **Houseplant authority sites** | 3-5k | houseplants | niche; small |

### To evaluate next
- Whether RHS / Garden.org / MOBOT offer downloadable lists or APIs, or whether the
  "cultivated realm" is only reachable via names against POWO + RHS filter scraping.

### Interpretation so far (educated truth)
- The corroborated cared-for set (~22k) mostly reflects **agri/food + germplasm + use**
  lists (GRIN/WCUPS-centric). **Ornamental/horticultural cultivation is under-represented**
  because RHS/Garden.org/MOBOT were never ingested. Adding ornamental-horticulture
  evidence is the single biggest lever on the true "human-cultivated species" number.

## Open research threads
- RHS Plant Finder bulk export / API; license.
- Garden.org data license (community, likely CC-ish? verify).
- MOBOT Plant Finder accessibility.
- Houseplant registry collation.