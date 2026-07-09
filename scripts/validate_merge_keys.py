#!/usr/bin/env python3
"""B2: report duplicate risk on scientific_name and identifier collisions."""
from __future__ import annotations

import json
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    db = ROOT / "data/botanica-cultivated-v0.1.duckdb"
    silver = ROOT / "data/silver_keep/species.parquet"
    con = duckdb.connect()
    if db.exists():
        con.execute(f"ATTACH '{str(db).replace(chr(92), '/')}' AS src (READ_ONLY)")
        sp, sid = "src.species", "src.species_identifiers"
    else:
        con.execute(
            f"CREATE VIEW species AS SELECT * FROM read_parquet('{(ROOT / 'data/silver/species.parquet').as_posix()}')"
        )
        con.execute(
            f"CREATE VIEW species_identifiers AS SELECT * FROM read_parquet('{(ROOT / 'data/silver/species_identifiers.parquet').as_posix()}')"
        )
        sp, sid = "species", "species_identifiers"

    dup_names = con.execute(
        f"""
        SELECT lower(trim(scientific_name)) n, count(*) c
        FROM {sp}
        WHERE scientific_name IS NOT NULL
        GROUP BY 1 HAVING count(*) > 1
        ORDER BY 2 DESC
        LIMIT 20
        """
    ).fetchall()
    dup_ids = con.execute(
        f"""
        SELECT source, external_id, count(*) c
        FROM {sid}
        GROUP BY 1, 2 HAVING count(*) > 1
        LIMIT 20
        """
    ).fetchall()
    report = {
        "policy": "Merge: (source, external_id) unique → else lower(scientific_name) → quarantine",
        "duplicate_scientific_names_top": [{"name": n, "count": c} for n, c in dup_names],
        "duplicate_identifier_pairs": [
            {"source": s, "external_id": e, "count": c} for s, e, c in dup_ids
        ],
        "ok_no_dup_identifiers": len(dup_ids) == 0,
        "dup_name_groups": len(dup_names),
    }
    out = ROOT / "data/manifests/merge-key-report.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2)[:1500])
    print(f"wrote {out}")
    # fail only on identifier collisions (true bug)
    return 0 if report["ok_no_dup_identifiers"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
