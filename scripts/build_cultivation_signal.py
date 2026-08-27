#!/usr/bin/env python3
"""Normalize all cultivation evidence into one standard signal mart.

Goal (data-science, not conjecture): every source that says "this species is
cultivated / used by humans / maintained" is pulled into a single normalized table
`cultivation_signal (species_id, signal_source, signal_kind, evidence, source_type)`.

Source classification:
  CULTIVATION  - list that asserts human cultivation/use:
       grin    (germplasm accessions)
       wcups   (Kew World Checklist of Useful Plant Species; + individual use-cats)
       itpgrfa (FAO treaty food/forage crops)
       faostat (FAO crop commodities)
  MAINTENANCE  - we keep care/maintenance data about the species:
       trait   (USDA HasChar trait rows)
       cultivation (cultivation_requirements rows)
  TAXONOMY     - name/occurrence backbone (NOT a cultivation vote):
       powo / usda / gbif / wikidata

Each signal source maps to a signal_kind:
  grin   -> germplasm
  wcups  -> human_use          (with sub-kind = use category when present)
  itpgrfa-> treaty_food_crop
  faostat-> crop_commodity
  trait  -> maintenance_trait
  cult   -> maintenance_req

Writes: data/gold/cultivation_signal.parquet (normalized, one row per signal)
        data/manifests/cultivation-signal.json (counts per source/kind)
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

# source -> (kind, type)
KIND = {
    "grin": ("germplasm", "CULTIVATION"),
    "wcups": ("human_use", "CULTIVATION"),
    "itpgrfa": ("treaty_food_crop", "CULTIVATION"),
    "faostat": ("crop_commodity", "CULTIVATION"),
    "trait": ("maintenance_trait", "MAINTENANCE"),
    "cultivation": ("maintenance_req", "MAINTENANCE"),
    "powo": ("taxonomy_names", "TAXONOMY"),
    "usda": ("taxonomy_flora", "TAXONOMY"),
    "gbif": ("taxonomy_occurrence", "TAXONOMY"),
    "wikidata": ("taxonomy_crossref", "TAXONOMY"),
}

# WCUPS use-code -> category label
WCUPS_CAT = {
    "AF": "animal_food",
    "EU": "environmental",
    "FU": "fuel",
    "GS": "gene_source",
    "HF": "human_food",
    "IF": "invertebrate_food",
    "MA": "material",
    "ME": "medicinal",
    "PO": "poison",
    "SU": "social",
}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="write parquet + json")
    args = ap.parse_args()
    con = duckdb.connect(str(DB))
    GOLD.mkdir(parents=True, exist_ok=True)
    MAN.mkdir(parents=True, exist_ok=True)

    # identifier-based signals
    ident_rows = con.execute(
        "SELECT species_id, lower(source) src FROM species_identifiers"
    ).fetchall()
    # trait / cultivation maintenance signals
    trait_sp = {
        r[0] for r in con.execute("SELECT DISTINCT species_id FROM traits").fetchall()
    }
    cult_sp = {
        r[0]
        for r in con.execute(
            "SELECT DISTINCT species_id FROM cultivation_requirements"
        ).fetchall()
    }

    # WCUPS use categories from the bronze extract
    import csv

    wcups_uses: dict[str, set] = {}
    tsv = ROOT / "data/bronze/allowlist/wcups/extracted/wcups_species.tsv"
    if tsv.exists():
        with open(tsv) as f:
            for row in csv.DictReader(f, delimiter="\t"):
                lsid = (row.get("lsid") or "").strip()
                for code in (row.get("uses") or "").split():
                    if code in WCUPS_CAT:
                        wcups_uses.setdefault(lsid, set()).add(WCUPS_CAT[code])
    # map wcups lsid -> species_id
    lsid2sp = {}
    for r in con.execute(
        "SELECT species_id, external_id FROM species_identifiers WHERE lower(source)='wcups'"
    ).fetchall():
        lsid2sp.setdefault((r[1] or "").strip(), r[0])

    rows = []
    for sid, src in ident_rows:
        kind, stype = KIND.get(src, ("unknown", "UNKNOWN"))
        rows.append((sid, src, kind, src, stype))
    for sid in trait_sp:
        rows.append((sid, "trait", "maintenance_trait", "trait", "MAINTENANCE"))
    for sid in cult_sp:
        rows.append(
            (sid, "cultivation", "maintenance_req", "cultivation", "MAINTENANCE")
        )
    for lsid, cats in wcups_uses.items():
        sid = lsid2sp.get(lsid)
        if not sid:
            continue
        for cat in cats:
            rows.append((sid, f"wcups:{cat}", cat, "wcups", "CULTIVATION"))

    # dedupe rows
    seen = set()
    uniq = []
    for r in rows:
        key = (r[0], r[1], r[2])
        if key not in seen:
            seen.add(key)
            uniq.append(r)

    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "n_signal_rows": len(uniq),
        "n_cultivation_species": len({r[0] for r in uniq if r[4] == "CULTIVATION"}),
        "n_maintenance_species": len({r[0] for r in uniq if r[4] == "MAINTENANCE"}),
        "by_source": {},
        "by_kind": {},
    }
    from collections import Counter

    cs = Counter(r[1] for r in uniq)
    ck = Counter(r[2] for r in uniq)
    report["by_source"] = dict(cs.most_common())
    report["by_kind"] = dict(ck.most_common())

    if args.apply:
        # write normalized parquet in one set-based pass (no per-row inserts)
        out = str((GOLD / "cultivation_signal.parquet").resolve()).replace("\\", "/")
        con.execute("DROP TABLE IF EXISTS cultivation_signal")
        con.execute(
            "CREATE TABLE cultivation_signal (species_id VARCHAR, signal_source VARCHAR, signal_kind VARCHAR, evidence VARCHAR, source_type VARCHAR)"
        )
        # register rows as a pandas-free parquet via temp values only if small;
        # use COPY from a derived query built by appending chunks
        import duckdb as _ddb

        tmp = _ddb.connect(":memory:")
        tmp.execute(
            "CREATE TABLE t (species_id VARCHAR, signal_source VARCHAR, signal_kind VARCHAR, evidence VARCHAR, source_type VARCHAR)"
        )
        chunk = []

        def flush():
            if not chunk:
                return
            vals = ",".join(
                f"('{c[0]}','{c[1]}','{c[2]}','{c[3]}','{c[4]}')" for c in chunk
            )
            tmp.execute(f"INSERT INTO t VALUES {vals}")
            chunk.clear()

        for sid, src, kind, evidence, stype in uniq:
            if any(ch in sid for ch in ("'", "\\")) or any(
                ch in src for ch in ("'", "\\")
            ):
                # parametrized fallback
                tmp.execute(
                    "INSERT INTO t VALUES (?,?,?,?,?)",
                    [sid, src, kind, evidence, stype],
                )
            else:
                chunk.append((sid, src, kind, evidence, stype))
            if len(chunk) >= 5000:
                flush()
        flush()
        p = str((GOLD / ".t.parquet").resolve()).replace("\\", "/")
        tmp.execute(f"COPY t TO '{p}' (FORMAT PARQUET)")
        con.execute(f"INSERT INTO cultivation_signal SELECT * FROM read_parquet('{p}')")
        con.execute(f"COPY cultivation_signal TO '{out}' (FORMAT PARQUET)")
        import os

        os.remove(p)
        (MAN / "cultivation-signal.json").write_text(
            json.dumps(report, indent=2), encoding="utf-8"
        )
        print(json.dumps(report, indent=2))
    else:
        print("DRY RUN (--apply to write)")
        print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
