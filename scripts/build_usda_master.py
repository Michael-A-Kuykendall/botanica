#!/usr/bin/env python3
"""Build master species list CSV from USDA PlantSearch catalog + genus→family map.

Output: data/bronze/usda_catalog/master_species.csv
Columns: scientific_name,symbol,family,genus,source,rank
"""
from __future__ import annotations

import argparse
import csv
import json
import re
from pathlib import Path


def strip_html(s: str) -> str:
    return re.sub(r"<[^>]+>", "", s or "").strip()


def parse_binomial(name: str):
    parts = name.split()
    if len(parts) < 2:
        return None
    return parts[0], parts[1], f"{parts[0]} {parts[1]}"


def clean_family(name: str) -> str:
    """Strip author strings like 'Liliaceae Juss.' → 'Liliaceae'."""
    if not name:
        return name
    # First token is family name for standard botanical families
    return name.split()[0].strip()


def load_family_map(paths) -> dict:
    m = {}
    for p in paths:
        path = Path(p)
        if not path.exists():
            continue
        with path.open(encoding="utf-8") as f:
            for row in csv.DictReader(f):
                if row.get("genus") and row.get("family"):
                    m[row["genus"]] = clean_family(row["family"])
    return m


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--catalog", default="data/bronze/usda_catalog/plant_search_pct.json")
    ap.add_argument("--out", default="data/bronze/usda_catalog/master_species.csv")
    ap.add_argument(
        "--ranks",
        default="Species",
        help="Comma ranks to include (default Species). Use Species,Subspecies,Variety for more.",
    )
    args = ap.parse_args()
    ranks = {r.strip() for r in args.ranks.split(",") if r.strip()}

    fam = load_family_map(
        [
            "data/lookups/genus_family.csv",
            "data/lookups/genus_family_usda.csv",
        ]
    )
    with open(args.catalog, encoding="utf-8") as f:
        rows = json.load(f)

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    accepted = quarantined = 0
    with out_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(
            f,
            fieldnames=["scientific_name", "symbol", "family", "genus", "source", "rank"],
        )
        w.writeheader()
        for row in rows:
            pl = row.get("Plant") or {}
            rank = pl.get("Rank") or ""
            if rank not in ranks:
                continue
            symbol = pl.get("Symbol") or ""
            raw = strip_html(pl.get("ScientificName") or "")
            parsed = parse_binomial(raw)
            if not symbol or not parsed:
                quarantined += 1
                continue
            genus, epithet, sci = parsed
            family = fam.get(genus)
            if not family:
                quarantined += 1
                continue
            w.writerow(
                {
                    "scientific_name": sci,
                    "symbol": symbol,
                    "family": family,
                    "genus": genus,
                    "source": "usda",
                    "rank": rank,
                }
            )
            accepted += 1

    print(f"wrote {out_path} accepted={accepted} skipped_no_family_or_parse={quarantined}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
