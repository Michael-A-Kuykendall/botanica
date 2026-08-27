#!/usr/bin/env python3
"""Ingest GRIN Taxonomy (GBIF-hosted dump) as free cultivated/germplasm allowlist.

Source: https://hosted-datasets.gbif.org/datasets/grin.zip (CC-friendly GRIN taxonomy)
Rule: species-rank rows with empty status = accepted GRIN species (synonyms have status set).

Writes:
  data/bronze/grin/grin_species_names.txt
  data/manifests/grin-allowlist.json
Marks matches on warehouse by scientific_name → species_identifiers source=grin
Expands KEEP membership note: GRIN hit OR cultivation payload.
"""
from __future__ import annotations

import argparse
import csv
import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]
GRIN_TSV = ROOT / "data/bronze/grin/extracted/NameUsage.tsv"


def load_grin_species() -> list[dict]:
    out = []
    with GRIN_TSV.open(encoding="utf-8", errors="replace", newline="") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            if (row.get("col:rank") or "").lower() != "species":
                continue
            status = (row.get("col:status") or "").strip().lower()
            if status:  # synonym
                continue
            name = (row.get("col:scientificName") or "").strip()
            if not name:
                continue
            gid = (row.get("col:ID") or "").strip()
            out.append(
                {
                    "grin_id": gid,
                    "scientific_name": name,
                    "authorship": (row.get("col:authorship") or "").strip(),
                    "link": (row.get("col:link") or "").strip(),
                }
            )
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--db",
        type=Path,
        default=ROOT / "data/botanica-cultivated-v0.1.duckdb",
    )
    ap.add_argument("--apply", action="store_true", help="Write grin identifiers into DuckDB")
    args = ap.parse_args()
    if not GRIN_TSV.exists():
        print("missing", GRIN_TSV, "download grin.zip first")
        return 2

    taxa = load_grin_species()
    print(f"GRIN accepted species names: {len(taxa)}")
    names_path = ROOT / "data/bronze/grin/grin_species_names.txt"
    names_path.write_text(
        "\n".join(t["scientific_name"] for t in taxa) + "\n", encoding="utf-8"
    )

    db = args.db if args.db.is_absolute() else ROOT / args.db
    con = duckdb.connect(str(db))
    # warehouse index
    idx = {
        n: sid
        for sid, n in con.execute(
            "SELECT id, lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL"
        ).fetchall()
        if n
    }
    matched = 0
    batch = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    if args.apply:
        for t in taxa:
            key = t["scientific_name"].lower().strip()
            sid = idx.get(key)
            if not sid:
                continue
            matched += 1
            gid = t["grin_id"] or key
            con.execute(
                """
                INSERT INTO species_identifiers
                SELECT ?, ?, 'grin', ?, 0, current_timestamp
                WHERE NOT EXISTS (
                  SELECT 1 FROM species_identifiers WHERE source='grin' AND external_id=?
                )
                """,
                [str(uuid.uuid4()), sid, gid, gid],
            )
        print(f"matched existing species and tagged grin: {matched}")
        # export identifiers + species silver warehouse
        for t in ("species_identifiers", "species"):
            p = str((ROOT / "data/silver" / f"{t}.parquet").resolve()).replace("\\", "/")
            con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
    else:
        for t in taxa:
            if t["scientific_name"].lower().strip() in idx:
                matched += 1
        print(f"would match (dry-run): {matched}")

    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "GRIN Taxonomy via GBIF hosted zip",
        "url": "https://hosted-datasets.gbif.org/datasets/grin.zip",
        "license": "USDA GRIN / checklist — redistribute with attribution (see GRIN/GBIF)",
        "grin_accepted_species": len(taxa),
        "matched_warehouse_species": matched,
        "batch": batch,
        "applied": bool(args.apply),
        "names_file": "data/bronze/grin/grin_species_names.txt",
    }
    out = ROOT / "data/manifests/grin-allowlist.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
