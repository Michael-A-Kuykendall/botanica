#!/usr/bin/env python3
"""C1: Wikidata hardiness (P8193/P8194) for KEEP species via SPARQL. CC0."""
from __future__ import annotations

import json
import time
import urllib.parse
import urllib.request
import uuid
from datetime import datetime, timezone
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]
UA = "Botanica/0.3 (OSS; github.com/Michael-A-Kuykendall/botanica)"

# https://www.wikidata.org/wiki/Property:P8194 hardiness zone
# https://www.wikidata.org/wiki/Property:P8193 hardiness of plant
P_ZONE = "P8194"
P_HARDY = "P8193"


def sparql(query: str) -> list[dict]:
    body = urllib.parse.urlencode({"query": query, "format": "json"}).encode()
    req = urllib.request.Request(
        "https://query.wikidata.org/sparql",
        data=body,
        headers={
            "User-Agent": UA,
            "Accept": "application/sparql-results+json",
            "Content-Type": "application/x-www-form-urlencoded",
        },
    )
    with urllib.request.urlopen(req, timeout=180) as r:
        j = json.loads(r.read().decode("utf-8"))
    return j.get("results", {}).get("bindings", [])


def label_value(binding_val: str) -> str:
    if not binding_val:
        return ""
    if binding_val.startswith("http"):
        return binding_val.rsplit("/", 1)[-1]
    return binding_val


def main() -> int:
    db = ROOT / "data/botanica-cultivated-v0.1.duckdb"
    con = duckdb.connect(str(db))
    keep_sp = ROOT / "data/silver_keep/species.parquet"
    names = [
        r[0]
        for r in con.execute(
            f"SELECT scientific_name FROM read_parquet('{keep_sp.as_posix()}') "
            f"WHERE scientific_name IS NOT NULL ORDER BY 1"
        ).fetchall()
    ]
    print(f"KEEP names={len(names)}")

    idx = {
        n: sid
        for sid, n in con.execute(
            "SELECT id, lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL"
        ).fetchall()
        if n
    }

    batch_size = 100
    zone_writes = 0
    hardy_writes = 0
    matched_taxa = 0
    with_any_zone = set()
    with_any_hardy = set()

    for i in range(0, len(names), batch_size):
        chunk = names[i : i + batch_size]
        vals = " ".join(json.dumps(n) for n in chunk)
        q = f"""
        SELECT ?taxonName ?taxon ?zone ?zoneLabel ?hardy ?hardyLabel WHERE {{
          VALUES ?taxonName {{ {vals} }}
          ?taxon wdt:P225 ?taxonName .
          OPTIONAL {{
            ?taxon wdt:{P_ZONE} ?zone .
            OPTIONAL {{ ?zone rdfs:label ?zoneLabel . FILTER(LANG(?zoneLabel)="en") }}
          }}
          OPTIONAL {{
            ?taxon wdt:{P_HARDY} ?hardy .
            OPTIONAL {{ ?hardy rdfs:label ?hardyLabel . FILTER(LANG(?hardyLabel)="en") }}
          }}
        }}
        """
        try:
            rows = sparql(q)
        except Exception as e:
            print(f"batch {i} fail {e}")
            time.sleep(8)
            continue

        for b in rows:
            matched_taxa += 1
            name = (b.get("taxonName") or {}).get("value") or ""
            sid = idx.get(name.lower().strip())
            if not sid:
                continue
            qid = ((b.get("taxon") or {}).get("value") or "").rsplit("/", 1)[-1]
            if qid:
                con.execute(
                    """
                    INSERT INTO species_identifiers
                    SELECT ?, ?, 'wikidata', ?, 0, current_timestamp
                    WHERE NOT EXISTS (
                      SELECT 1 FROM species_identifiers WHERE source='wikidata' AND external_id=?
                    )
                    """,
                    [str(uuid.uuid4()), sid, qid, qid],
                )

            zone = (b.get("zoneLabel") or {}).get("value") or label_value(
                (b.get("zone") or {}).get("value") or ""
            )
            hardy = (b.get("hardyLabel") or {}).get("value") or label_value(
                (b.get("hardy") or {}).get("value") or ""
            )
            if zone:
                con.execute(
                    """
                    INSERT INTO cultivation_requirements
                    (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability, created_at)
                    SELECT ?, ?, 'hardiness_zone', ?, NULL, NULL, 'wikidata P8194', 'WIKIDATA', 1, current_timestamp
                    WHERE NOT EXISTS (
                      SELECT 1 FROM cultivation_requirements
                      WHERE species_id=? AND requirement_type='hardiness_zone' AND source='WIKIDATA'
                        AND value_text=?
                    )
                    """,
                    [str(uuid.uuid4()), sid, zone, sid, zone],
                )
                zone_writes += 1
                with_any_zone.add(sid)
            if hardy:
                con.execute(
                    """
                    INSERT INTO cultivation_requirements
                    (id, species_id, requirement_type, value_text, value_numeric, units, notes, source, reliability, created_at)
                    SELECT ?, ?, 'hardiness', ?, NULL, NULL, 'wikidata P8193', 'WIKIDATA', 1, current_timestamp
                    WHERE NOT EXISTS (
                      SELECT 1 FROM cultivation_requirements
                      WHERE species_id=? AND requirement_type='hardiness' AND source='WIKIDATA'
                        AND value_text=?
                    )
                    """,
                    [str(uuid.uuid4()), sid, hardy, sid, hardy],
                )
                hardy_writes += 1
                with_any_hardy.add(sid)

        print(
            f"  {i+len(chunk)}/{len(names)} sparql={len(rows)} "
            f"zone_spp={len(with_any_zone)} hardy_spp={len(with_any_hardy)}",
            flush=True,
        )
        time.sleep(0.8)

    for t in ("cultivation_requirements", "species_identifiers"):
        p = str((ROOT / "data/silver" / f"{t}.parquet").resolve()).replace("\\", "/")
        con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")

    sun_n = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM cultivation_requirements
        WHERE lower(requirement_type) IN ('sunlight','light')
        """
    ).fetchone()[0]

    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "Wikidata SPARQL P8194 hardiness zone / P8193 hardiness of plant",
        "license": "CC0",
        "keep_names": len(names),
        "sparql_taxon_matches": matched_taxa,
        "species_with_hardiness_zone": len(with_any_zone),
        "species_with_hardiness": len(with_any_hardy),
        "zone_rows_written": zone_writes,
        "hardy_rows_written": hardy_writes,
        "species_with_sunlight_any_source": sun_n,
        "sunlight_note": "No dedicated free bulk sunlight property used; USDA sunlight still sparse",
        "ceiling": True,
    }
    out = ROOT / "data/manifests/wikidata-hardiness.json"
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
