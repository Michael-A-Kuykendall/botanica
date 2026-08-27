#!/usr/bin/env python3
"""FAOSTAT crop commodity list → match EN vernaculars → tag species source=faostat.

Not scientific names — commodity labels (Apples, Barley, …). Match via vernacular_names.
"""

from __future__ import annotations

import csv
import json
import re
import uuid
from datetime import datetime, timezone
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]
ITEMS = (
    ROOT / "data/bronze/faostat/extracted/Production_Crops_Livestock_E_ItemCodes.csv"
)


def clean_item(item: str) -> list[str]:
    """Return candidate common-name strings from FAO item label."""
    item = item.strip()
    if not item:
        return []
    # drop livestock/animal-ish crude
    low = item.lower()
    skip = (
        "meat",
        "milk",
        "butter",
        "cheese",
        "egg",
        "honey",
        "bees",
        "cattle",
        "sheep",
        "goat",
        "pig",
        "chicken",
        "duck",
        "turkey",
        "horse",
        "camel",
        "buffalo",
        "asses",
        "mules",
        "skins",
        "hides",
        "wool",
        "silk",
        "fat",
        "lard",
        "tallow",
        "offals",
        "snails",
        "meat",
    )
    if any(s in low for s in skip) and "bean" not in low:
        # allow "Bambara beans" etc.
        if not any(
            x in low
            for x in (
                "bean",
                "nut",
                "seed",
                "fruit",
                "berry",
                "grape",
                "rice",
                "wheat",
                "maize",
                "potato",
                "tomato",
                "onion",
                "apple",
                "banana",
                "citrus",
            )
        ):
            if re.search(
                r"\b(meat|milk|butter|cheese|egg|cattle|sheep|pig|chicken)\b", low
            ):
                return []
    primary = item.split(";")[0].strip()
    # strip ", dry" ", raw" style tails after comma if short
    primary = re.sub(
        r",\s*(dry|green|raw|fresh|in shell).*$", "", primary, flags=re.I
    ).strip()
    cands = {primary, primary.lower()}
    # first token for multi-word if useful
    if " " in primary:
        cands.add(primary.split()[0])
    return [c for c in cands if len(c) > 2]


def main() -> int:
    if not ITEMS.exists():
        print("missing", ITEMS)
        return 2
    items = []
    with ITEMS.open(encoding="latin-1", newline="") as f:
        for row in csv.DictReader(f):
            items.append(row["Item"])
    cands = set()
    for it in items:
        for c in clean_item(it):
            cands.add(c.lower())
    print(f"faostat items={len(items)} name_candidates={len(cands)}")

    db = ROOT / "data/botanica-cultivated-v0.1.duckdb"
    con = duckdb.connect(str(db))
    # match vernacular en
    vern = con.execute(
        """
        SELECT species_id, lower(trim(name)) AS n
        FROM vernacular_names
        WHERE lower(language_code) IN ('en','eng','en-us','en-gb')
        """
    ).fetchall()
    hit_species = set()
    for sid, n in vern:
        if n in cands:
            hit_species.add(sid)
        # plural rough: apples -> apple
        if n.endswith("s") and n[:-1] in cands:
            hit_species.add(sid)

    batch = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    for sid in hit_species:
        con.execute(
            """
            INSERT INTO species_identifiers
            SELECT ?, ?, 'faostat', ?, 0, current_timestamp
            WHERE NOT EXISTS (
              SELECT 1 FROM species_identifiers WHERE source='faostat' AND external_id=? AND species_id=?
            )
            """,
            [str(uuid.uuid4()), sid, sid[:16], sid[:16], sid],
        )
    print(f"tagged species via vernacular match: {len(hit_species)}")
    p = str((ROOT / "data/silver/species_identifiers.parquet").resolve()).replace(
        "\\", "/"
    )
    con.execute(f"COPY (SELECT * FROM species_identifiers) TO '{p}' (FORMAT PARQUET)")
    from shard_parquet import shard as shard_out, TARGET_MB_DEFAULT

    shard_out(ROOT / "data/silver", TARGET_MB_DEFAULT, con)

    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "FAOSTAT Production Crops Livestock item codes",
        "url": "https://fenixservices.fao.org/faostat/static/bulkdownloads/Production_Crops_Livestock_E_All_Data.zip",
        "license": "FAOSTAT — free for use with attribution",
        "commodity_items": len(items),
        "species_tagged": len(hit_species),
        "batch": batch,
        "note": "Match is EN vernacular ≈ commodity label; not Latin binomial map",
    }
    out = ROOT / "data/manifests/faostat-allowlist.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    # save candidate names
    (ROOT / "data/bronze/faostat/crop_name_candidates.txt").write_text(
        "\n".join(sorted(cands)) + "\n", encoding="utf-8"
    )
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
