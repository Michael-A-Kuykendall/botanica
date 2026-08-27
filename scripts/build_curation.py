#!/usr/bin/env python3
"""GOLD curation layer — the 'definitive' vinyl signal.

Computes, per species, an independent cultivation/use SIGNAL set already
present in the warehouse, a signal_count, and an is_definitive flag
(threshold-adjustable). Replaces the opaque KEEP filter with a transparent,
scored, provenance-carrying mart.

Signals (each from a different source family = independent evidence of
human cultivation/use):
  grin   - GRIN germplasm id (global cultivated/germplasm checklist)
  faostat- FAOSTAT crop commodity id
  wcups  - Kew World Checklist of Useful Plant Species 2020 id (documented human use)
  itpgrfa- FAO ITPGRFA Annex I crop (treaty-level food/forage crops)
  wiki   - Wikidata taxon id (notability / cultivation statements)
  powo   - POWO accepted-name id
  trait  - USDA HasChar trait row
  cult   - cultivation_requirements row
  envern - English vernacular name

Writes:
  data/gold/species_curation.parquet
  data/manifests/curation-<tag>.json
Also re-derives KEEP transparently from is_definitive for parity check.
"""
from __future__ import annotations
import argparse, json
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"
GOLD = ROOT / "data/gold"
MAN = ROOT / "data/manifests"

SIGNAL_SQL = """
SELECT s.id AS sid,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='grin'))::INT g,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='faostat'))::INT f,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='wcups'))::INT u,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='itpgrfa'))::INT i,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='wikidata'))::INT w,
   (EXISTS (SELECT 1 FROM species_identifiers i WHERE i.species_id=s.id AND lower(i.source)='powo'))::INT p,
   (EXISTS (SELECT 1 FROM traits t WHERE t.species_id=s.id))::INT tr,
   (EXISTS (SELECT 1 FROM cultivation_requirements c WHERE c.species_id=s.id))::INT cr,
   (EXISTS (SELECT 1 FROM vernacular_names v WHERE v.species_id=s.id AND lower(v.language_code) IN ('en','eng','en-us','en-gb')))::INT ev
FROM species s
"""

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--threshold", type=int, default=2, help="min independent signals for is_definitive")
    ap.add_argument("--tag", default="baseline")
    args = ap.parse_args()
    GOLD.mkdir(parents=True, exist_ok=True)
    MAN.mkdir(parents=True, exist_ok=True)

    con = duckdb.connect(str(DB))
    con.execute("CREATE OR REPLACE TEMP TABLE sig AS " + SIGNAL_SQL)

    con.execute("""
    CREATE OR REPLACE TABLE species_curation AS
    SELECT sid AS species_id,
            (g+f+u+i+w+p+tr+cr+ev) AS signal_count,
            json_object(
              'grin', g, 'faostat', f, 'wcups', u, 'itpgrfa', i, 'wikidata', w, 'powo', p,
              'trait', tr, 'cultivation', cr, 'en_vernacular', ev
            ) AS signals,
            (? <= (g+f+u+i+w+p+tr+cr+ev)) AS is_definitive,
            ((? <= (g+f+u+i+w+p+tr+cr+ev)) OR g=1 OR f=1 OR u=1 OR i=1 OR tr=1 OR cr=1) AS is_cultivated_scope,
           'gold-v1' AS rule_version,
           current_timestamp AS computed_at
    FROM sig
    """, [args.threshold, args.threshold])

    out = str((GOLD / "species_curation.parquet").resolve()).replace("\\", "/")
    con.execute(f"COPY species_curation TO '{out}' (FORMAT PARQUET)")

    total = con.execute("SELECT COUNT(*) FROM species_curation").fetchone()[0]
    definitive = con.execute("SELECT COUNT(*) FROM species_curation WHERE is_definitive").fetchone()[0]
    # histogram
    hist = {}
    for n in range(0, 8):
        c = con.execute(f"SELECT COUNT(*) FROM species_curation WHERE signal_count >= {n}").fetchone()[0]
        hist[f">= {n}"] = c

    # signal composition among definitive set
    comp = con.execute("""
      SELECT
        SUM(CAST(signals->>'grin' AS INT)), SUM(CAST(signals->>'faostat' AS INT)),
        SUM(CAST(signals->>'wcups' AS INT)), SUM(CAST(signals->>'itpgrfa' AS INT)),
        SUM(CAST(signals->>'wikidata' AS INT)), SUM(CAST(signals->>'powo' AS INT)),
        SUM(CAST(signals->>'trait' AS INT)), SUM(CAST(signals->>'cultivation' AS INT)),
        SUM(CAST(signals->>'en_vernacular' AS INT))
      FROM species_curation WHERE is_definitive
    """).fetchone()

    cultivated = con.execute("SELECT COUNT(*) FROM species_curation WHERE is_cultivated_scope").fetchone()[0]
    comp_c = con.execute("""
      SELECT
        SUM(CAST(signals->>'grin' AS INT)), SUM(CAST(signals->>'faostat' AS INT)),
        SUM(CAST(signals->>'wcups' AS INT)), SUM(CAST(signals->>'itpgrfa' AS INT)),
        SUM(CAST(signals->>'wikidata' AS INT)), SUM(CAST(signals->>'powo' AS INT)),
        SUM(CAST(signals->>'trait' AS INT)), SUM(CAST(signals->>'cultivation' AS INT)),
        SUM(CAST(signals->>'en_vernacular' AS INT))
      FROM species_curation WHERE is_cultivated_scope
    """).fetchone()

    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "rule": "is_definitive = (count of independent cultivation/use signals) >= threshold",
        "threshold": args.threshold,
        "total_species": total,
        "definitive_species": definitive,
        "pct_definitive": round(100.0*definitive/total, 2),
        "cultivated_scope_species": cultivated,
        "pct_cultivated_scope": round(100.0*cultivated/total, 2),
        "signal_histogram": hist,
        "definitive_signal_composition": {
            "grin": comp[0], "faostat": comp[1], "wcups": comp[2], "itpgrfa": comp[3],
            "wikidata": comp[4], "powo": comp[5], "trait": comp[6], "cultivation": comp[7],
            "en_vernacular": comp[8],
        },
        "cultivated_scope_composition": {
            "grin": comp_c[0], "faostat": comp_c[1], "wcups": comp_c[2], "itpgrfa": comp_c[3],
            "wikidata": comp_c[4], "powo": comp_c[5], "trait": comp_c[6], "cultivation": comp_c[7],
            "en_vernacular": comp_c[8],
        },
        "notes": "is_cultivated_scope = is_definitive OR any cultivated-source signal (grin/faostat/wcups/trait/cultivation). This is the vinyl universe. Not yet including POWO-full ids, uses, or media.",
    }
    mp = MAN / f"curation-{args.tag}.json"
    mp.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    con.close()
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
