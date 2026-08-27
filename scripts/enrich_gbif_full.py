#!/usr/bin/env python3
"""Memory-conscious GBIF full enrich into DuckDB + silver export."""
from __future__ import annotations

import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    db = ROOT / "data/botanica-cultivated-v0.1.duckdb"
    norm_dir = ROOT / "data/bronze/gbif_vern/full/normalized"
    norms = sorted(norm_dir.glob("GBIF_norm_*.json"))
    if not norms:
        print("no GBIF norm")
        return 2
    path = norms[-1]
    print(f"loading {path} mb={path.stat().st_size / 1e6:.1f}")
    con = duckdb.connect(str(db))
    idx = {
        n: sid
        for sid, n in con.execute(
            "SELECT id, lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL"
        ).fetchall()
        if n
    }
    print("species idx", len(idx))
    batch = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    counts = {"matched": 0, "vern": 0, "en": 0, "skip": 0}

    # 81MB is fine for json.load
    recs = json.loads(path.read_text(encoding="utf-8"))
    print("records", len(recs))

    for i, rec in enumerate(recs, 1):
        q = (rec.get("query_name") or "").strip().lower()
        if not q or not rec.get("matched"):
            counts["skip"] += 1
            continue
        sid = idx.get(q) or idx.get((rec.get("canonical_name") or "").strip().lower())
        if not sid:
            counts["skip"] += 1
            continue
        counts["matched"] += 1
        gkey = rec.get("gbif_key")
        if gkey is not None:
            gk = str(gkey)
            con.execute(
                """
                INSERT INTO species_identifiers
                SELECT ?, ?, 'gbif', ?, 0, current_timestamp
                WHERE NOT EXISTS (
                  SELECT 1 FROM species_identifiers WHERE source='gbif' AND external_id=?
                )
                """,
                [str(uuid.uuid4()), sid, gk, gk],
            )
        for v in rec.get("vernacular_names") or []:
            name = (v.get("name") or "").strip()
            lang = (v.get("language") or "und").strip() or "und"
            if not name:
                continue
            primary = 1 if v.get("preferred") else 0
            con.execute(
                """
                INSERT INTO vernacular_names
                (id, species_id, name, language_code, is_primary, source, created_at)
                SELECT ?, ?, ?, ?, ?, 'GBIF', current_timestamp
                WHERE NOT EXISTS (
                  SELECT 1 FROM vernacular_names
                  WHERE species_id=? AND name=? AND language_code=? AND source='GBIF'
                )
                """,
                [str(uuid.uuid4()), sid, name, lang, primary, sid, name, lang],
            )
            counts["vern"] += 1
            if lang.lower() in ("en", "eng", "en-us", "en-gb"):
                counts["en"] += 1
        con.execute(
            """
            INSERT INTO provenance
            (id, species_id, source, source_record_id, license, retrieved_at, hash)
            VALUES (?, ?, 'GBIF', ?, 'CC BY 4.0', current_timestamp, ?)
            """,
            [str(uuid.uuid4()), sid, str(gkey or q), batch],
        )
        if i % 2000 == 0:
            print(f"  {i}/{len(recs)} {counts}", flush=True)

    print("DONE", counts)
    silver = ROOT / "data/silver"
    for t in (
        "vernacular_names",
        "species_identifiers",
        "provenance",
        "synonyms",
        "traits",
        "distribution_regions",
        "uses",
        "species",
        "cultivation_requirements",
    ):
        p = str((silver / f"{t}.parquet").resolve()).replace("\\", "/")
        try:
            con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
            print("exported", t)
        except Exception as e:
            print("skip", t, e)

    en = con.execute(
        """
        SELECT COUNT(DISTINCT species_id) FROM vernacular_names
        WHERE lower(language_code) IN ('en','eng','en-us','en-gb')
        """
    ).fetchone()[0]
    sp = con.execute("SELECT COUNT(*) FROM species").fetchone()[0]
    print(f"en_species={en} / {sp} = {100*en/sp:.2f}%")

    mpath = ROOT / "data/manifests/botanica-cultivated-v0.1.json"
    if mpath.exists():
        man = json.loads(mpath.read_text(encoding="utf-8"))
        man["built_at"] = datetime.now(timezone.utc).isoformat()
        man["counts"] = {
            "families": con.execute("SELECT COUNT(*) FROM families").fetchone()[0],
            "genera": con.execute("SELECT COUNT(*) FROM genera").fetchone()[0],
            "species": sp,
            "cultivars": con.execute("SELECT COUNT(*) FROM cultivars").fetchone()[0],
            "traits": con.execute("SELECT COUNT(*) FROM traits").fetchone()[0],
            "vernacular_names": con.execute(
                "SELECT COUNT(*) FROM vernacular_names"
            ).fetchone()[0],
            "cultivation_requirements": con.execute(
                "SELECT COUNT(*) FROM cultivation_requirements"
            ).fetchone()[0],
            "provenance": con.execute("SELECT COUNT(*) FROM provenance").fetchone()[0],
            "quarantine": con.execute(
                "SELECT COUNT(*) FROM ingest_quarantine"
            ).fetchone()[0],
            "plants": con.execute("SELECT COUNT(*) FROM plants").fetchone()[0],
            "synonyms": con.execute("SELECT COUNT(*) FROM synonyms").fetchone()[0],
            "uses": con.execute("SELECT COUNT(*) FROM uses").fetchone()[0],
            "distribution_regions": con.execute(
                "SELECT COUNT(*) FROM distribution_regions"
            ).fetchone()[0],
        }
        man.setdefault("sources", []).append(
            {
                "name": "GBIF_VERNACULAR_FULL",
                "license": "CC BY 4.0",
                "record_count": counts["matched"],
                "notes": f"vern={counts['vern']} en_rows={counts['en']} batch={batch}",
            }
        )
        man["scope"] = (man.get("scope") or "") + f" + GBIF full vern batch={batch}"
        man["notes"] = (man.get("notes") or "") + f" | GBIF full {counts}"
        mpath.write_text(json.dumps(man, indent=2), encoding="utf-8")
        print("manifest updated")
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
