#!/usr/bin/env python3
"""V4/V5 (resumed) — WCVP bulk ingest: POWO-backbone enrichment for the warehouse.

Why WCVP instead of the POWO api:
  powo.science.kew.org/api/2 is behind a Cloudflare "Just a moment" challenge that 403s
  from this host (even with a browser UA). Kew publishes the exact backbone data as a bulk
  Darwin-Core archive: sftp.kew.org/pub/data-repositories/WCVP/wcvp_dwca.zip (2026-06).
  WCVP is the same checklist POWO's API serves, with the same IPNI LSIDs, plus
  lifeform/climate (dynamicproperties) and TDWG distribution.

Delivers (vs V4/V5 beads):
  - species_identifiers source='powo'   (accepted-name LSIDs, consistent w/ existing 2.6k)
  - synonyms     source='powo'          (from acceptednameusageid closure)
  - distribution_regions source='powo'  (TDWG WGSRPD areas)
  - lifeform / climate                   (dynamicproperties), reported in manifest

Fail-fast: debug (5 known) -> smoke (first 1000 of scope + coverage) -> full (--apply).
Idempotent: existing powo identifiers / synonym rows / dist rows are skipped.
"""

from __future__ import annotations
import argparse, json, re, uuid
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"
WCVP = ROOT / "data/bronze/wcvp"

TAXON_COLS = {
    "taxonid": "VARCHAR",
    "family": "VARCHAR",
    "genus": "VARCHAR",
    "specificepithet": "VARCHAR",
    "infraspecificepithet": "VARCHAR",
    "scientfiicname": "VARCHAR",
    "scientfiicnameauthorship": "VARCHAR",
    "taxonrank": "VARCHAR",
    "taxonomicstatus": "VARCHAR",
    "acceptednameusageid": "VARCHAR",
    "parentnameusageid": "VARCHAR",
    "originalnameusageid": "VARCHAR",
    "namepublishedin": "VARCHAR",
    "nomenclaturalstatus": "VARCHAR",
    "taxonremarks": "VARCHAR",
    "scientificnameid": "VARCHAR",
    "dynamicproperties": "VARCHAR",
    "references": "VARCHAR",
}
DIST_COLS = {
    "coreid": "VARCHAR",
    "locality": "VARCHAR",
    "establishmentmeans": "VARCHAR",
    "locationid": "VARCHAR",
    "occurrencestatus": "VARCHAR",
    "threatstatus": "VARCHAR",
}


def load_views(con: duckdb.DuckDBPyConnection) -> None:
    """Load WCVP raw CSVs into temp tables once."""
    con.execute(f"""
      CREATE OR REPLACE TEMP TABLE wcvp AS
      SELECT * FROM read_csv('{WCVP / "wcvp_taxon.csv"}',
        delim='|', header=true, nullstr='', auto_detect=false,
        columns={json.dumps(TAXON_COLS)})""")
    con.execute(f"""
      CREATE OR REPLACE TEMP TABLE wcvp_dist AS
      SELECT * FROM read_csv('{WCVP / "wcvp_distribution.csv"}',
        delim='|', header=true, nullstr='', auto_detect=false,
        columns={json.dumps(DIST_COLS)})""")
    # accepted species-level, one per binomial
    con.execute("""
      CREATE OR REPLACE TEMP TABLE wcvp_acc AS
      SELECT taxonid, family, genus, specificepithet, taxonrank, dynamicproperties
      FROM wcvp
      WHERE taxonomicstatus='Accepted' AND taxonrank='Species' AND specificepithet<>''
      QUALIFY row_number() OVER (PARTITION BY lower(genus), lower(specificepithet) ORDER BY taxonid)=1
    """)


def species_binom(con: duckdb.DuckDBPyConnection) -> None:
    con.execute("""
      CREATE OR REPLACE TEMP TABLE mysp AS
      SELECT s.id AS species_id, lower(trim(g.name)||' '||trim(s.specific_epithet)) AS binom
      FROM species s JOIN genera g ON s.genus_id=g.id
      WHERE s.specific_epithet IS NOT NULL AND trim(s.specific_epithet)<>''
      QUALIFY row_number() OVER (PARTITION BY lower(trim(g.name)||' '||trim(s.specific_epithet)) ORDER BY s.id)=1
    """)


def matched(con: duckdb.DuckDBPyConnection) -> duckdb.DuckDBPyConnection:
    con.execute("""
      CREATE OR REPLACE TEMP TABLE m AS
      SELECT m.species_id, w.taxonid, w.family,
             json_extract_string(w.dynamicproperties,'$.powoid') AS powoid,
             json_extract_string(w.dynamicproperties,'$.lifeform') AS lifeform,
             json_extract_string(w.dynamicproperties,'$.climate') AS climate
      FROM mysp m JOIN wcvp_acc w
        ON m.binom = lower(trim(w.genus)||' '||trim(w.specificepithet))
    """)
    return con


def report(con, scope_total: int) -> dict:
    n = con.execute("SELECT count(*) FROM m").fetchone()[0]
    if n == 0:
        return {"matched": 0}
    lf = con.execute(
        "SELECT count(*) FROM m WHERE lifeform IS NOT NULL AND lifeform<>''"
    ).fetchone()[0]
    cl = con.execute(
        "SELECT count(*) FROM m WHERE climate IS NOT NULL AND climate<>''"
    ).fetchone()[0]
    pid = con.execute(
        "SELECT count(*) FROM m WHERE powoid IS NOT NULL AND powoid<>''"
    ).fetchone()[0]
    # distribution coverage (rows in wcvp_dist for matched taxonids)
    d = con.execute(
        "SELECT count(DISTINCT coreid) FROM wcvp_dist WHERE coreid IN (SELECT taxonid FROM m)"
    ).fetchone()[0]
    return {
        "scope_species": scope_total,
        "matched": n,
        "pct_matched": round(100.0 * n / scope_total, 2),
        "pct_with_powoid": round(100.0 * pid / n, 2),
        "pct_lifeform": round(100.0 * lf / n, 2),
        "pct_climate": round(100.0 * cl / n, 2),
        "species_with_distribution": d,
        "pct_with_distribution": round(100.0 * d / n, 2),
    }


def apply_all(con: duckdb.DuckDBPyConnection) -> dict:
    """Insert identifiers, synonyms, distribution into the DB (idempotent)."""
    # 1) powo identifiers (LSID form, matching existing 2.6k). Unique on (source, external_id):
    #    keep the first species per LSID, skip species that already carry a powo id.
    con.execute("""
      INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at)
      SELECT uuid(), species_id, 'powo', lsid, 0, current_timestamp
      FROM (
        SELECT species_id, 'urn:lsid:ipni.org:names:'||powoid AS lsid,
               row_number() OVER (PARTITION BY powoid ORDER BY species_id) AS rn
        FROM m WHERE powoid IS NOT NULL AND powoid<>''
      ) q
      WHERE q.rn = 1
        AND NOT EXISTS (SELECT 1 FROM species_identifiers i
                        WHERE lower(i.source)='powo' AND i.species_id=q.species_id)
        AND NOT EXISTS (SELECT 1 FROM species_identifiers i
                        WHERE i.source='powo' AND i.external_id=q.lsid)
    """)
    ident = con.execute("SELECT count(*) FROM m").fetchall()[0][0]  # informational
    # 2) synonyms = wcvp rows whose acceptednameusageid is one of our accepted taxonids
    con.execute("""
      INSERT INTO synonyms (id, species_id, synonym_name, authorship, source, source_record_id, created_at)
      SELECT uuid(), species_id, synonym_name, authorship, 'powo', source_record_id, current_timestamp
      FROM (
        SELECT m.species_id, w.scientfiicname AS synonym_name,
               w.scientfiicnameauthorship AS authorship, w.taxonid AS source_record_id,
               row_number() OVER (PARTITION BY m.species_id, lower(w.scientfiicname) ORDER BY w.taxonid) AS rn
        FROM wcvp w
        JOIN m ON w.acceptednameusageid = m.taxonid
        WHERE w.taxonid <> m.taxonid AND w.scientfiicname IS NOT NULL AND trim(w.scientfiicname)<>''
      ) q
      WHERE q.rn = 1
        AND NOT EXISTS (SELECT 1 FROM synonyms sy
                        WHERE sy.species_id=q.species_id AND sy.source='powo'
                          AND sy.synonym_name=q.synonym_name)
    """)
    # 3) distribution_regions from wcvp_dist (TDWG areas)
    con.execute("""
      INSERT INTO distribution_regions (id, species_id, region_code, region_source, notes, source, created_at)
      SELECT uuid(), species_id, region_code, 'WGSRPD', notes, 'powo', current_timestamp
      FROM (
        SELECT m.species_id, d.locationid AS region_code, d.locality AS notes,
               row_number() OVER (PARTITION BY m.species_id, d.locationid ORDER BY d.locality) AS rn
        FROM wcvp_dist d
        JOIN m ON d.coreid = m.taxonid
        WHERE d.locationid IS NOT NULL AND trim(d.locationid)<>''
      ) q
      WHERE q.rn = 1
        AND NOT EXISTS (SELECT 1 FROM distribution_regions dr
                        WHERE dr.species_id=q.species_id AND dr.source='powo'
                          AND dr.region_code=q.region_code)
    """)
    return {
        "created_at": datetime.now(timezone.utc).isoformat(),
        "note": "use manifest counters for exact row deltas",
    }


def export_silver(con: duckdb.DuckDBPyConnection) -> None:
    from shard_parquet import shard as shard_out, TARGET_MB_DEFAULT

    for t in ("species_identifiers", "synonyms", "distribution_regions", "species"):
        p = str((ROOT / "data/silver" / f"{t}.parquet").resolve()).replace("\\", "/")
        con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
    shard_out(ROOT / "data/silver", TARGET_MB_DEFAULT, con)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", choices=["debug", "smoke", "full"], default="debug")
    ap.add_argument("--apply", action="store_true", help="write DB rows (full only)")
    ap.add_argument("--tag", default="wcvp_v1")
    args = ap.parse_args()

    con = duckdb.connect(str(DB))
    load_views(con)
    species_binom(con)

    if args.phase == "debug":
        for name in (
            "Quercus alba",
            "Aloe vera",
            "Camellia sinensis",
            "Zea mays",
            "Solanum lycopersicum",
        ):
            hit = con.execute(
                "SELECT taxonid, family, dynamicproperties FROM wcvp_acc WHERE lower(genus||' '||specificepithet)=?",
                [name.lower()],
            ).fetchone()
            print(f"  {name:24} -> {hit}")
        return 0

    scope = "species" if args.phase == "full" else "sample"
    if args.phase == "smoke":
        # scope = first 1000 species (by id) from mysp
        con.execute("""
          CREATE OR REPLACE TEMP TABLE scope_sp AS
          SELECT * FROM mysp ORDER BY species_id LIMIT 1000""")
        con.execute("CREATE OR REPLACE TEMP TABLE mysp AS SELECT * FROM scope_sp")

    scope_total = con.execute("SELECT count(*) FROM mysp").fetchall()[0][0]
    matched(con)
    rep = report(con, scope_total)
    rep.update({"phase": args.phase, "tag": args.tag, "scope": scope})
    print(json.dumps(rep, indent=2))

    if args.phase == "full":
        # re-scope to the whole species set (smoke narrowed mysp)
        species_binom(con)
        matched(con)
        scope_total = con.execute("SELECT count(*) FROM mysp").fetchall()[0][0]
        if args.apply:
            apply_all(con)
            export_silver(con)
            print("DB written + silver parquet exported")
        (ROOT / "data/manifests" / f"{args.tag}-ingest.json").write_text(
            json.dumps(report(con, scope_total), indent=2), encoding="utf-8"
        )
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
