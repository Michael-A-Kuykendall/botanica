# USDA PLANTS → Botanica field map (locked)

Version: **1.0** (2026-07-09). Source scraper: `botanica_usda` normalized JSON.

| USDA / normalized key | Botanica table.column | Notes |
|----------------------|------------------------|-------|
| `usda_symbol` | `species_identifiers` source=`usda` | Primary merge key |
| `scientific_name` | `species.scientific_name` | Binomial parse → genus + epithet |
| `common_names[]` | `vernacular_names` lang=`en` source=`USDA` | First is primary |
| `horticultural_traits.plant_type` | `traits.growth_habit` | joined `; ` |
| `horticultural_traits.duration` | `traits.duration` | |
| `horticultural_traits.mature_height_cm` | `traits.mature_height` numeric m | cm→m |
| `horticultural_traits.toxicity` | `traits.toxicity` | |
| `horticultural_traits.sunlight` | `cultivation_requirements.sunlight` | often empty; shade_tolerance separate |
| `horticultural_traits.soil` | `cultivation_requirements.soil` | practical Tier1 |
| `horticultural_traits.moisture` | `cultivation_requirements.moisture` | practical Tier1 |
| `ecological_traits.shade_tolerance` | `traits.shade_tolerance` | |
| `ecological_traits.drought_tolerance` | `traits.drought_tolerance` | |
| `ecological_traits.salt_tolerance` | `traits.salt_tolerance` | |
| `ecological_traits.wetland_indicator` | `traits.wetland_indicator` | |
| `distributions.native[]` | `distribution_regions` | source USDA |
| `distributions.introduced[]` | `distribution_regions` | |

## Practical Tier1 (coverage gates)

`soil`, `moisture`, `mature_height`, `toxicity`

Do **not** gate full runs on USDA hardiness (absent in Characteristics path).

## Provenance

Each enrich batch → `provenance` with source `USDA_*` and payload hash when available.
