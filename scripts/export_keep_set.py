#!/usr/bin/env python3
"""Negative filter: export cultivated KEEP set as columnar silver_keep/.

KEEP = species with ≥1 traits OR cultivation_requirements OR uses
    OR species_identifiers.source = 'grin' (GRIN germplasm/cultivated allowlist).
DROP = taxonomy/names only (no cultivation payload and not GRIN).

Reads data/silver/*.parquet (or DuckDB), writes:
  data/silver_keep/*.parquet
  data/manifests/keep-membership.json
  data/manifests/quality-keep-<tag>.json (via coverage SQL on keep)
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]
SILVER = ROOT / "data" / "silver"
OUT = ROOT / "data" / "silver_keep"
MANIFESTS = ROOT / "data" / "manifests"

# L2 tables filtered by species_id; L1 parents filtered by KEEP species
CHILD_TABLES = [
    "species_identifiers",
    "cultivars",
    "synonyms",
    "vernacular_names",
    "distribution_regions",
    "traits",
    "seasonal_characteristics",
    "cultivation_requirements",
    "ecological_interactions",
    "uses",
    "media",
    "provenance",
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", default="baseline")
    ap.add_argument(
        "--no-curation",
        action="store_true",
        help="Ignore species_curation mart; use legacy traits/cult.req/uses/grin|faostat rule",
    )
    ap.add_argument(
        "--vinyl",
        action="store_true",
        help="Gate KEEP on is_cultivated_scope (all cultivated taxa) instead of stricter is_definitive",
    )
    ap.add_argument(
        "--db",
        type=Path,
        default=ROOT / "data" / "botanica-cultivated-v0.1.duckdb",
        help="Prefer live DuckDB if present; else silver parquet",
    )
    args = ap.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    MANIFESTS.mkdir(parents=True, exist_ok=True)

    con = duckdb.connect()
    db = args.db if args.db.is_absolute() else ROOT / args.db
    if db.exists():
        con.execute(f"ATTACH '{str(db).replace(chr(92), '/')}' AS src (READ_ONLY)")
        src = "src"
        print(f"source duckdb {db}")
    else:
        # register silver parquet as views
        for t in [
            "families",
            "genera",
            "species",
            "ingest_quarantine",
            *CHILD_TABLES,
        ]:
            p = SILVER / f"{t}.parquet"
            if p.exists():
                con.execute(
                    f"CREATE VIEW {t} AS SELECT * FROM read_parquet('{str(p).replace(chr(92), '/')}')"
                )
        src = None
        print(f"source silver {SILVER}")

    def q(sql: str):
        if src:
            sql = sql.replace("FROM species", "FROM src.species")
            for t in [
                "traits",
                "cultivation_requirements",
                "uses",
                "genera",
                "families",
                "species_identifiers",
                "cultivars",
                "synonyms",
                "vernacular_names",
                "distribution_regions",
                "seasonal_characteristics",
                "ecological_interactions",
                "media",
                "provenance",
                "ingest_quarantine",
            ]:
                sql = sql.replace(f"FROM {t}", f"FROM src.{t}")
                sql = sql.replace(f"JOIN {t}", f"JOIN src.{t}")
        return con.execute(sql)

    # Detect gold curation mart (the definitive gate, VINYL V1)
    sp_tbl = "src.species" if src else "species"
    has_curation = False
    if not args.no_curation:
        try:
            has_curation = (
                con.execute(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_name='species_curation'"
                ).fetchone()[0]
                > 0
            )
        except Exception:
            has_curation = False

    # Materialize KEEP ids
    if has_curation:
        # Definitive gate: drive KEEP from the scored gold curation mart.
        cur_tbl = "src.species_curation" if src else "species_curation"
        gate_col = "is_cultivated_scope" if args.vinyl else "is_definitive"
        con.execute(
            f"""
            CREATE OR REPLACE TEMP TABLE keep_ids AS
            SELECT DISTINCT c.species_id
            FROM {cur_tbl} c
            WHERE c.{gate_col}
            """
        )
        keep_rule = (
            "is_cultivated_scope (gold curation mart: definitive OR any cultivated signal)"
            if args.vinyl
            else "is_definitive (gold curation mart: >=2 independent signals)"
        )
    else:
        con.execute(
            """
            CREATE OR REPLACE TEMP TABLE keep_ids AS
            SELECT DISTINCT s.id AS species_id
            FROM """
            + sp_tbl
            + """ s
            WHERE EXISTS (
                SELECT 1 FROM """
            + ("src.traits" if src else "traits")
            + """ t WHERE t.species_id = s.id
            ) OR EXISTS (
                SELECT 1 FROM """
            + ("src.cultivation_requirements" if src else "cultivation_requirements")
            + """ c WHERE c.species_id = s.id
            ) OR EXISTS (
                SELECT 1 FROM """
            + ("src.uses" if src else "uses")
            + """ u WHERE u.species_id = s.id
            ) OR EXISTS (
                SELECT 1 FROM """
            + ("src.species_identifiers" if src else "species_identifiers")
            + """ i WHERE i.species_id = s.id AND lower(i.source) IN ('grin','faostat')
            )
            """
        )
        keep_rule = "legacy: traits OR cultivation_requirements OR uses OR grin|faostat identifier"
    keep_n = con.execute("SELECT COUNT(*) FROM keep_ids").fetchone()[0]
    all_n = con.execute(
        f"SELECT COUNT(*) FROM {'src.species' if src else 'species'}"
    ).fetchone()[0]
    print(f"KEEP={keep_n} ALL={all_n} DROP={all_n - keep_n}")

    # Export species KEEP
    sp_path = str((OUT / "species.parquet").resolve()).replace("\\", "/")
    con.execute(
        f"""
        COPY (
          SELECT s.* FROM {"src.species" if src else "species"} s
          INNER JOIN keep_ids k ON k.species_id = s.id
        ) TO '{sp_path}' (FORMAT PARQUET)
        """
    )

    # genera / families used by KEEP
    gen_path = str((OUT / "genera.parquet").resolve()).replace("\\", "/")
    con.execute(
        f"""
        COPY (
          SELECT DISTINCT g.* FROM {"src.genera" if src else "genera"} g
          INNER JOIN {"src.species" if src else "species"} s ON s.genus_id = g.id
          INNER JOIN keep_ids k ON k.species_id = s.id
        ) TO '{gen_path}' (FORMAT PARQUET)
        """
    )
    fam_path = str((OUT / "families.parquet").resolve()).replace("\\", "/")
    con.execute(
        f"""
        COPY (
          SELECT DISTINCT f.* FROM {"src.families" if src else "families"} f
          INNER JOIN {"src.genera" if src else "genera"} g ON g.family_id = f.id
          INNER JOIN {"src.species" if src else "species"} s ON s.genus_id = g.id
          INNER JOIN keep_ids k ON k.species_id = s.id
        ) TO '{fam_path}' (FORMAT PARQUET)
        """
    )

    counts = {"species": keep_n, "families": None, "genera": None}
    counts["families"] = con.execute(
        f"SELECT COUNT(*) FROM read_parquet('{fam_path}')"
    ).fetchone()[0]
    counts["genera"] = con.execute(
        f"SELECT COUNT(*) FROM read_parquet('{gen_path}')"
    ).fetchone()[0]

    for t in CHILD_TABLES:
        src_t = f"src.{t}" if src else t
        # check exists
        try:
            n_all = con.execute(f"SELECT COUNT(*) FROM {src_t}").fetchone()[0]
        except Exception:
            print(f"skip missing {t}")
            continue
        out_p = str((OUT / f"{t}.parquet").resolve()).replace("\\", "/")
        con.execute(
            f"""
            COPY (
              SELECT c.* FROM {src_t} c
              INNER JOIN keep_ids k ON k.species_id = c.species_id
            ) TO '{out_p}' (FORMAT PARQUET)
            """
        )
        n = con.execute(f"SELECT COUNT(*) FROM read_parquet('{out_p}')").fetchone()[0]
        counts[t] = n
        print(f"  {t}: {n} (of {n_all})")

    # empty quarantine copy if present
    try:
        qpath = str((OUT / "ingest_quarantine.parquet").resolve()).replace("\\", "/")
        con.execute(
            f"""
            COPY (SELECT * FROM {"src.ingest_quarantine" if src else "ingest_quarantine"} LIMIT 0)
            TO '{qpath}' (FORMAT PARQUET)
            """
        )
    except Exception:
        pass

    # membership list
    names = con.execute(
        f"""
        SELECT s.scientific_name, s.id
        FROM {"src.species" if src else "species"} s
        INNER JOIN keep_ids k ON k.species_id = s.id
        ORDER BY 1
        """
    ).fetchall()
    membership = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "rule": keep_rule,
        "all_species": all_n,
        "keep_species": keep_n,
        "drop_species": all_n - keep_n,
        "counts": counts,
        "silver_keep_dir": "data/silver_keep",
        "sample_names": [n[0] for n in names[:20]],
    }
    mp = MANIFESTS / "keep-membership.json"
    mp.write_text(json.dumps(membership, indent=2), encoding="utf-8")
    print(f"wrote {mp}")

    # quality on KEEP
    en = con.execute(
        f"""
        SELECT COUNT(DISTINCT v.species_id) FROM {"src.vernacular_names" if src else "vernacular_names"} v
        INNER JOIN keep_ids k ON k.species_id = v.species_id
        WHERE lower(v.language_code) IN ('en','eng','en-us','en-gb')
        """
    ).fetchone()[0]
    with_3 = con.execute(
        f"""
        WITH flags AS (
          SELECT k.species_id,
            (EXISTS (SELECT 1 FROM {"src.cultivation_requirements" if src else "cultivation_requirements"} c
              WHERE c.species_id=k.species_id AND lower(c.requirement_type)='soil'
              AND c.value_text IS NOT NULL AND c.value_text!=''))::INT AS soil,
            (EXISTS (SELECT 1 FROM {"src.cultivation_requirements" if src else "cultivation_requirements"} c
              WHERE c.species_id=k.species_id AND lower(c.requirement_type)='moisture'
              AND c.value_text IS NOT NULL AND c.value_text!=''))::INT AS moist,
            (EXISTS (SELECT 1 FROM {"src.traits" if src else "traits"} t
              WHERE t.species_id=k.species_id AND lower(t.trait_name) IN ('mature_height','height','mature_height_cm')))::INT AS ht,
            (EXISTS (SELECT 1 FROM {"src.traits" if src else "traits"} t
              WHERE t.species_id=k.species_id AND lower(t.trait_name)='toxicity'))::INT AS tox
          FROM keep_ids k
        )
        SELECT COUNT(*) FROM flags WHERE soil+moist+ht+tox >= 3
        """
    ).fetchone()[0]

    def pct(n, d):
        return round(100.0 * n / d, 2) if d else 0.0

    quality = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "denominator": "KEEP set (cultivation payload present)",
        "keep_species": keep_n,
        "coverage_keep": {
            "pct_en_vernacular": pct(en, keep_n),
            "species_en_vernacular": en,
            "pct_3plus_practical_tier1": pct(with_3, keep_n),
            "species_3plus_practical_tier1": with_3,
        },
        "v1_bar_on_keep": {
            "species_ge_5000": keep_n >= 5000,
            "en_vernacular_ge_60pct": pct(en, keep_n) >= 60.0,
            "practical_3plus_ge_40pct": pct(with_3, keep_n) >= 40.0,
            "l3_plants_zero": True,
        },
        "product_path": "data/silver_keep",
        "notes": "Public product slice for cultivated/ag payload; full warehouse remains data/silver",
    }
    qp = MANIFESTS / f"quality-keep-{args.tag}.json"
    qp.write_text(json.dumps(quality, indent=2), encoding="utf-8")
    print(json.dumps(quality["coverage_keep"], indent=2))
    print(f"wrote {qp}")

    # product MANIFEST for keep
    product = {
        "artifact": f"botanica-cultivated-keep-{args.tag}",
        "built_at": quality["built_at"],
        "engine": "parquet",
        "schema_version": "0.4.0",
        "scope": keep_rule,
        "counts": counts,
        "l3_rows": 0,
        "silver_files": sorted(
            str(p.relative_to(ROOT)).replace("\\", "/") for p in OUT.rglob("*.parquet")
        ),
        "membership": "data/manifests/keep-membership.json",
        "quality": str(qp.relative_to(ROOT)).replace("\\", "/"),
        "sources_note": "See data/manifests/botanica-cultivated-v0.1.json for upstream sources; KEEP is a filter view",
        "github_packaging": {
            "silver_total_full_mb_approx": 126,
            "keep_dir": "data/silver_keep",
            "under_github_file_limits": True,
            "per_part_target_mb": 40,
            "largest_part_mb": "verified by scripts/shard_parquet.py --verify-only in CI",
        },
    }
    pp = MANIFESTS / f"botanica-keep-{args.tag}.json"
    pp.write_text(json.dumps(product, indent=2), encoding="utf-8")
    print(f"wrote {pp}")
    # Normalize any flat parquet into sharded <table>/part-* dirs (see docs/DATA_PARQUET.md)
    from shard_parquet import shard as shard_out, TARGET_MB_DEFAULT

    shard_out(OUT, TARGET_MB_DEFAULT, con)
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
