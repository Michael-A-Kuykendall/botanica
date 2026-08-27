#!/usr/bin/env python3
"""Export scientific name lists for POWO/GBIF scrapes from DuckDB seed."""
from __future__ import annotations

import argparse
from pathlib import Path

import duckdb


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", type=Path, default=Path("data/botanica-cultivated-v0.1.duckdb"))
    ap.add_argument("--out-dir", type=Path, default=Path("data/bronze/name_lists"))
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1]
    db = args.db if args.db.is_absolute() else root / args.db
    out = args.out_dir if args.out_dir.is_absolute() else root / args.out_dir
    out.mkdir(parents=True, exist_ok=True)
    con = duckdb.connect(str(db), read_only=True)

    all_names = con.execute(
        """
        SELECT scientific_name FROM species
        WHERE scientific_name IS NOT NULL AND trim(scientific_name) != ''
        ORDER BY scientific_name
        """
    ).fetchall()
    (out / "all_species_names.txt").write_text(
        "\n".join(r[0] for r in all_names) + "\n", encoding="utf-8"
    )

    # species with any trait/req = haschar proxy / hort dense set
    dense = con.execute(
        """
        SELECT DISTINCT s.scientific_name
        FROM species s
        WHERE s.scientific_name IS NOT NULL
          AND (
            EXISTS (SELECT 1 FROM traits t WHERE t.species_id = s.id)
            OR EXISTS (SELECT 1 FROM cultivation_requirements c WHERE c.species_id = s.id)
          )
        ORDER BY 1
        """
    ).fetchall()
    (out / "has_trait_names.txt").write_text(
        "\n".join(r[0] for r in dense) + "\n", encoding="utf-8"
    )

    # missing English vernacular
    missing_en = con.execute(
        """
        SELECT s.scientific_name FROM species s
        WHERE s.scientific_name IS NOT NULL AND trim(s.scientific_name) != ''
          AND NOT EXISTS (
            SELECT 1 FROM vernacular_names v
            WHERE v.species_id = s.id
              AND lower(v.language_code) IN ('en','eng','en-us','en-gb')
          )
        ORDER BY 1
        """
    ).fetchall()
    (out / "missing_en_vernacular.txt").write_text(
        "\n".join(r[0] for r in missing_en) + "\n", encoding="utf-8"
    )

    # stratified sample 1000: every Nth from all
    step = max(1, len(all_names) // 1000)
    sample = [all_names[i][0] for i in range(0, len(all_names), step)][:1000]
    (out / "sample_1000.txt").write_text("\n".join(sample) + "\n", encoding="utf-8")

    print(
        f"all={len(all_names)} dense={len(dense)} missing_en={len(missing_en)} sample={len(sample)} → {out}"
    )
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
