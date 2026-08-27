#!/usr/bin/env python3
"""Enrich existing DuckDB seed from POWO/GBIF normalized bronze JSON.

Matches on scientific_name (case-insensitive). Writes identifiers, synonyms,
distribution, traits (lifeform/climate), uses (if any), vernaculars, provenance.
Re-exports silver parquet tables and updates MANIFEST counts lightly.
"""
from __future__ import annotations

import argparse
import json
import uuid
from datetime import datetime, timezone
from pathlib import Path

import duckdb


def latest_norm(dir_path: Path, prefix: str) -> Path | None:
    if not dir_path.exists():
        return None
    candidates = []
    for p in dir_path.rglob("*.json"):
        if "norm" in p.name.lower() and prefix.lower() in p.name.lower():
            candidates.append(p)
        elif "norm" in p.name.lower() and dir_path.name.startswith(prefix.lower()[:4]):
            candidates.append(p)
    # prefer under normalized/
    norms = [p for p in candidates if "normalized" in str(p).replace("\\", "/")]
    pool = norms or candidates
    if not pool:
        return None
    pool.sort(key=lambda p: p.stat().st_mtime)
    return pool[-1]


def load_json(path: Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, list):
        return data
    if isinstance(data, dict) and "records" in data:
        return data["records"]
    return [data]


def species_index(con) -> dict[str, str]:
    rows = con.execute(
        "SELECT id, lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL"
    ).fetchall()
    return {name: sid for sid, name in rows if name}


def insert_powo(con, rec: dict, sid: str, batch: str) -> dict:
    stats = {"syn": 0, "loc": 0, "trait": 0, "use": 0, "ident": 0}
    powo_id = rec.get("powo_id")
    if powo_id:
        con.execute(
            """
            INSERT INTO species_identifiers
            SELECT ?, ?, 'powo', ?, 0, current_timestamp
            WHERE NOT EXISTS (
              SELECT 1 FROM species_identifiers WHERE source='powo' AND external_id=?
            )
            """,
            [str(uuid.uuid4()), sid, powo_id, powo_id],
        )
        stats["ident"] = 1
    for syn in rec.get("synonyms") or []:
        name = (syn.get("name") or "").strip()
        if not name:
            continue
        auth = syn.get("author") or ""
        con.execute(
            """
            INSERT INTO synonyms (id, species_id, synonym_name, authorship, source, source_record_id)
            SELECT ?, ?, ?, ?, 'POWO', ?
            WHERE NOT EXISTS (
              SELECT 1 FROM synonyms WHERE species_id=? AND synonym_name=? AND source='POWO'
            )
            """,
            [str(uuid.uuid4()), sid, name, auth, syn.get("fqId") or "", sid, name],
        )
        stats["syn"] += 1
    for loc in rec.get("locations") or []:
        code = str(loc).strip()
        if not code:
            continue
        con.execute(
            """
            INSERT INTO distribution_regions (id, species_id, region_code, region_source, notes, source)
            SELECT ?, ?, ?, 'WGSRPD', NULL, 'POWO'
            WHERE NOT EXISTS (
              SELECT 1 FROM distribution_regions WHERE species_id=? AND region_code=? AND source='POWO'
            )
            """,
            [str(uuid.uuid4()), sid, code, sid, code],
        )
        stats["loc"] += 1
    for tname, val in (("lifeform", rec.get("lifeform")), ("climate", rec.get("climate"))):
        if not val:
            continue
        con.execute(
            """
            INSERT INTO traits
            (id, species_id, trait_name, trait_value_text, trait_value_numeric, units, method, source, reliability, created_at)
            SELECT ?, ?, ?, ?, NULL, NULL, NULL, 'POWO', 1, current_timestamp
            WHERE NOT EXISTS (
              SELECT 1 FROM traits WHERE species_id=? AND trait_name=? AND source='POWO'
            )
            """,
            [str(uuid.uuid4()), sid, tname, str(val), sid, tname],
        )
        stats["trait"] += 1
    for u in rec.get("uses") or []:
        cat = u.get("category") or "unspecified"
        desc = u.get("description") or ""
        con.execute(
            """
            INSERT INTO uses (id, species_id, use_category, description, source, created_at)
            VALUES (?, ?, ?, ?, 'POWO', current_timestamp)
            """,
            [str(uuid.uuid4()), sid, cat, desc],
        )
        stats["use"] += 1
    con.execute(
        """
        INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
        VALUES (?, ?, 'POWO', ?, 'CC BY 4.0', current_timestamp, ?)
        """,
        [str(uuid.uuid4()), sid, powo_id or rec.get("query_name") or "", batch],
    )
    return stats


def insert_gbif(con, rec: dict, sid: str, batch: str) -> dict:
    stats = {"vern": 0, "ident": 0, "en": 0}
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
        stats["ident"] = 1
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
        stats["vern"] += 1
        if lang.lower() in ("en", "eng", "en-us", "en-gb"):
            stats["en"] += 1
    con.execute(
        """
        INSERT INTO provenance (id, species_id, source, source_record_id, license, retrieved_at, hash)
        VALUES (?, ?, 'GBIF', ?, 'CC BY 4.0', current_timestamp, ?)
        """,
        [str(uuid.uuid4()), sid, str(gkey or rec.get("query_name") or ""), batch],
    )
    return stats


def export_silver(con, silver_dir: Path) -> list[str]:
    silver_dir.mkdir(parents=True, exist_ok=True)
    tables = [
        "families",
        "genera",
        "species",
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
        "ingest_quarantine",
    ]
    written = []
    for t in tables:
        path = silver_dir / f"{t}.parquet"
        p = str(path).replace("\\", "/")
        try:
            con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
            written.append(p)
        except Exception as e:
            print(f"skip {t}: {e}")
    return written


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", type=Path, default=Path("data/botanica-cultivated-v0.1.duckdb"))
    ap.add_argument("--powo", type=Path, default=None, help="POWO norm JSON or dir")
    ap.add_argument("--gbif", type=Path, default=None, help="GBIF norm JSON or dir")
    ap.add_argument("--export-silver", action="store_true")
    ap.add_argument("--manifest", type=Path, default=Path("data/manifests/botanica-cultivated-v0.1.json"))
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1]
    db = args.db if args.db.is_absolute() else root / args.db
    con = duckdb.connect(str(db))
    idx = species_index(con)
    batch = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    totals = {"powo_matched": 0, "gbif_matched": 0, "syn": 0, "loc": 0, "trait": 0, "vern": 0, "en": 0}

    if args.powo:
        p = args.powo if args.powo.is_absolute() else root / args.powo
        path = p if p.is_file() else latest_norm(p, "POWO")
        if path:
            print(f"POWO enrich {path}")
            for rec in load_json(path):
                q = (rec.get("query_name") or rec.get("accepted_name") or "").strip().lower()
                if not q or not rec.get("matched"):
                    continue
                sid = idx.get(q) or idx.get((rec.get("accepted_name") or "").strip().lower())
                if not sid:
                    continue
                st = insert_powo(con, rec, sid, batch)
                totals["powo_matched"] += 1
                totals["syn"] += st["syn"]
                totals["loc"] += st["loc"]
                totals["trait"] += st["trait"]
        else:
            print("no POWO norm found")

    if args.gbif:
        p = args.gbif if args.gbif.is_absolute() else root / args.gbif
        path = p if p.is_file() else latest_norm(p, "GBIF")
        if path:
            print(f"GBIF enrich {path}")
            for rec in load_json(path):
                q = (rec.get("query_name") or "").strip().lower()
                if not q or not rec.get("matched"):
                    continue
                sid = idx.get(q) or idx.get((rec.get("canonical_name") or "").strip().lower())
                if not sid:
                    continue
                st = insert_gbif(con, rec, sid, batch)
                totals["gbif_matched"] += 1
                totals["vern"] += st["vern"]
                totals["en"] += st["en"]
        else:
            print("no GBIF norm found")

    print("totals", totals)
    if args.export_silver:
        written = export_silver(con, root / "data" / "silver")
        print(f"exported {len(written)} silver files")
        # light MANIFEST count refresh
        mpath = args.manifest if args.manifest.is_absolute() else root / args.manifest
        if mpath.exists():
            man = json.loads(mpath.read_text(encoding="utf-8"))
            man["built_at"] = datetime.now(timezone.utc).isoformat()
            man["counts"] = {
                "families": con.execute("SELECT COUNT(*) FROM families").fetchone()[0],
                "genera": con.execute("SELECT COUNT(*) FROM genera").fetchone()[0],
                "species": con.execute("SELECT COUNT(*) FROM species").fetchone()[0],
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
            man["scope"] = (
                man.get("scope", "")
                + f" + POWO/GBIF enrich batch={batch} powo={totals['powo_matched']} gbif={totals['gbif_matched']}"
            )
            sources = man.get("sources") or []
            if totals["powo_matched"]:
                sources.append(
                    {
                        "name": "POWO",
                        "license": "CC BY 4.0",
                        "record_count": totals["powo_matched"],
                        "notes": f"syn={totals['syn']} loc={totals['loc']} trait={totals['trait']} batch={batch}",
                    }
                )
            if totals["gbif_matched"]:
                sources.append(
                    {
                        "name": "GBIF",
                        "license": "CC BY 4.0",
                        "record_count": totals["gbif_matched"],
                        "notes": f"vern={totals['vern']} en_rows≈{totals['en']} batch={batch}",
                    }
                )
            man["sources"] = sources
            man["notes"] = (
                (man.get("notes") or "")
                + f" | enrich {batch}: powo_matched={totals['powo_matched']} gbif_matched={totals['gbif_matched']}"
            )
            mpath.write_text(json.dumps(man, indent=2), encoding="utf-8")
            print(f"updated {mpath}")
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
