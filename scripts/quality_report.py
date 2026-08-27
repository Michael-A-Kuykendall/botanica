#!/usr/bin/env python3
"""Quality scoreboard for Botanica silver / DuckDB seed.

Produces data/manifests/quality-<tag>.json with measurable coverage metrics.
Denominators:
  all_species     — every species row
  has_any_trait   — species with ≥1 trait OR cult.req row
  haschar_proxy   — species with ≥1 of soil/moisture/height/toxicity (practical Tier1 set)

Practical Tier1: soil, moisture, mature_height (trait mature_height), toxicity
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

import duckdb


PRACTICAL = ("soil", "moisture", "mature_height", "toxicity")


def report(db_path: Path) -> dict:
    con = duckdb.connect(str(db_path), read_only=True)
    species_n = con.execute("SELECT COUNT(*) FROM species").fetchone()[0]
    plants_n = con.execute("SELECT COUNT(*) FROM plants").fetchone()[0]
    cultivars_n = con.execute("SELECT COUNT(*) FROM cultivars").fetchone()[0]

    # species with English vernacular
    en_vern = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM vernacular_names
        WHERE lower(language_code) IN ('en', 'eng', 'en-us', 'en-gb')
        """
    ).fetchone()[0]

    any_vern = con.execute(
        "SELECT COUNT(DISTINCT species_id) FROM vernacular_names"
    ).fetchone()[0]

    # practical Tier1 via traits + cultivation_requirements
    # soil/moisture/sunlight often in cultivation_requirements.requirement_type
    # toxicity, mature_height in traits.trait_name
    soil_s = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM cultivation_requirements
        WHERE lower(requirement_type) = 'soil' AND value_text IS NOT NULL AND value_text != ''
        """
    ).fetchone()[0]
    moisture_s = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM cultivation_requirements
        WHERE lower(requirement_type) = 'moisture' AND value_text IS NOT NULL AND value_text != ''
        """
    ).fetchone()[0]
    sun_s = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM cultivation_requirements
        WHERE lower(requirement_type) IN ('sunlight','light') AND value_text IS NOT NULL AND value_text != ''
        """
    ).fetchone()[0]
    height_s = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM traits
        WHERE lower(trait_name) IN ('mature_height','height','mature_height_cm')
        """
    ).fetchone()[0]
    tox_s = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM traits
        WHERE lower(trait_name) = 'toxicity'
          AND (trait_value_text IS NOT NULL AND trait_value_text != '' OR trait_value_numeric IS NOT NULL)
        """
    ).fetchone()[0]

    with_3plus = con.execute(
        """
        WITH flags AS (
          SELECT s.id AS species_id,
            (EXISTS (
              SELECT 1 FROM cultivation_requirements c
              WHERE c.species_id = s.id AND lower(c.requirement_type)='soil'
                AND c.value_text IS NOT NULL AND c.value_text != ''
            ))::INT AS has_soil,
            (EXISTS (
              SELECT 1 FROM cultivation_requirements c
              WHERE c.species_id = s.id AND lower(c.requirement_type)='moisture'
                AND c.value_text IS NOT NULL AND c.value_text != ''
            ))::INT AS has_moisture,
            (EXISTS (
              SELECT 1 FROM traits t
              WHERE t.species_id = s.id
                AND lower(t.trait_name) IN ('mature_height','height','mature_height_cm')
            ))::INT AS has_height,
            (EXISTS (
              SELECT 1 FROM traits t
              WHERE t.species_id = s.id AND lower(t.trait_name)='toxicity'
            ))::INT AS has_tox
          FROM species s
        )
        SELECT COUNT(*) FROM flags
        WHERE (has_soil + has_moisture + has_height + has_tox) >= 3
        """
    ).fetchone()[0]

    any_trait = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM (
          SELECT species_id FROM traits
          UNION
          SELECT species_id FROM cultivation_requirements
        )
        """
    ).fetchone()[0]

    uses_s = con.execute(
        "SELECT COUNT(DISTINCT species_id) FROM uses"
    ).fetchone()[0]
    uses_rows = con.execute("SELECT COUNT(*) FROM uses").fetchone()[0]
    synonyms_s = con.execute(
        "SELECT COUNT(DISTINCT species_id) FROM synonyms"
    ).fetchone()[0]
    dist_s = con.execute(
        "SELECT COUNT(DISTINCT species_id) FROM distribution_regions"
    ).fetchone()[0]

    by_source = con.execute(
        """
        SELECT source, COUNT(*) AS n FROM provenance GROUP BY 1 ORDER BY 2 DESC
        """
    ).fetchall()

    trait_names = con.execute(
        """
        SELECT trait_name, COUNT(*) FROM traits GROUP BY 1 ORDER BY 2 DESC LIMIT 30
        """
    ).fetchall()
    req_types = con.execute(
        """
        SELECT requirement_type, COUNT(*) FROM cultivation_requirements GROUP BY 1 ORDER BY 2 DESC
        """
    ).fetchall()

    def pct(n: int, d: int) -> float:
        return round(100.0 * n / d, 2) if d else 0.0

    out = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "db": str(db_path).replace("\\", "/"),
        "denominators": {
            "all_species": {
                "definition": "COUNT(*) FROM species",
                "n": species_n,
            },
            "has_any_trait": {
                "definition": "species with ≥1 traits OR cultivation_requirements row",
                "n": any_trait,
            },
            "haschar_proxy": {
                "definition": "species with ≥1 practical Tier1 field (soil|moisture|height|toxicity)",
                "n": con.execute(
                    """
                    SELECT COUNT(*) FROM (
                      SELECT s.id FROM species s
                      WHERE EXISTS (
                        SELECT 1 FROM cultivation_requirements c
                        WHERE c.species_id=s.id AND lower(c.requirement_type) IN ('soil','moisture')
                      ) OR EXISTS (
                        SELECT 1 FROM traits t
                        WHERE t.species_id=s.id
                          AND lower(t.trait_name) IN ('mature_height','height','toxicity','mature_height_cm')
                      )
                    )
                    """
                ).fetchone()[0],
            },
        },
        "counts": {
            "species": species_n,
            "plants_l3": plants_n,
            "cultivars": cultivars_n,
            "traits_rows": con.execute("SELECT COUNT(*) FROM traits").fetchone()[0],
            "cultivation_requirements_rows": con.execute(
                "SELECT COUNT(*) FROM cultivation_requirements"
            ).fetchone()[0],
            "vernacular_rows": con.execute("SELECT COUNT(*) FROM vernacular_names").fetchone()[0],
            "uses_rows": uses_rows,
            "synonyms_species": synonyms_s,
            "distribution_species": dist_s,
            "provenance_rows": con.execute("SELECT COUNT(*) FROM provenance").fetchone()[0],
        },
        "coverage_all_species": {
            "pct_en_vernacular": pct(en_vern, species_n),
            "pct_any_vernacular": pct(any_vern, species_n),
            "pct_any_trait": pct(any_trait, species_n),
            "pct_3plus_practical_tier1": pct(with_3plus, species_n),
            "pct_uses": pct(uses_s, species_n),
            "species_en_vernacular": en_vern,
            "species_any_vernacular": any_vern,
            "species_any_trait": any_trait,
            "species_3plus_practical_tier1": with_3plus,
            "species_uses": uses_s,
        },
        "coverage_has_any_trait": {
            "n": any_trait,
            "pct_3plus_practical_tier1": pct(with_3plus, any_trait),
            "pct_en_vernacular": pct(
                con.execute(
                    """
                    SELECT COUNT(DISTINCT v.species_id) FROM vernacular_names v
                    WHERE lower(v.language_code) IN ('en','eng','en-us','en-gb')
                      AND v.species_id IN (
                        SELECT species_id FROM traits
                        UNION SELECT species_id FROM cultivation_requirements
                      )
                    """
                ).fetchone()[0],
                any_trait,
            ),
        },
        "field_null_rates_practical": {
            "soil_species": soil_s,
            "moisture_species": moisture_s,
            "sunlight_species": sun_s,
            "mature_height_species": height_s,
            "toxicity_species": tox_s,
            "pct_soil": pct(soil_s, species_n),
            "pct_moisture": pct(moisture_s, species_n),
            "pct_sunlight": pct(sun_s, species_n),
            "pct_mature_height": pct(height_s, species_n),
            "pct_toxicity": pct(tox_s, species_n),
        },
        "by_source_provenance": [
            {"source": s, "record_count": n} for s, n in by_source
        ],
        "trait_name_histogram": [{"name": n, "count": c} for n, c in trait_names],
        "requirement_type_histogram": [{"name": n, "count": c} for n, c in req_types],
        "v1_bar_check": {
            "species_ge_5000": species_n >= 5000,
            "en_vernacular_ge_60pct": pct(en_vern, species_n) >= 60.0,
            "practical_3plus_ge_40pct": pct(with_3plus, species_n) >= 40.0,
            "l3_plants_zero": plants_n == 0,
            "notes": (
                "Global bar uses all_species denominator. "
                "USDA HasChar subset is much denser; see coverage_has_any_trait."
            ),
        },
        "practical_tier1_fields": list(PRACTICAL),
    }
    con.close()
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--db",
        type=Path,
        default=Path("data/botanica-cultivated-v0.1.duckdb"),
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Output JSON path (default data/manifests/quality-<ts>.json)",
    )
    ap.add_argument("--tag", default="baseline", help="filename tag")
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1]
    db = args.db if args.db.is_absolute() else root / args.db
    if not db.exists():
        print(f"missing db {db}", file=sys.stderr)
        return 2
    rep = report(db)
    out = args.out
    if out is None:
        out = root / "data" / "manifests" / f"quality-{args.tag}.json"
    elif not out.is_absolute():
        out = root / out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(rep, indent=2), encoding="utf-8")
    print(json.dumps(rep["coverage_all_species"], indent=2))
    print("v1_bar", json.dumps(rep["v1_bar_check"], indent=2))
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
