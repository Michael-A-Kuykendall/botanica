#!/usr/bin/env python3
"""VINYL V2 — Backbone: insert GRIN accepted species NOT already in the warehouse.

Source: GRIN Taxonomy (GBIF-hosted zip), already local at
        data/bronze/grin/extracted/NameUsage.tsv  (CC-friendly / USDA GRIN)
Rule:   species-rank rows with empty status = accepted GRIN species.
        For names NOT present in the warehouse, INSERT new species rows and
        resolve genus/family from GRIN itself (genus.parentID -> family.name),
        never inventing a family (architecture: no Unknown taxa).

Signals: each new species also gets a species_identifiers row source='grin',
         which the gold curation mart counts as a cultivation signal.

Fail-fast: default --dry-run prints counts only. --apply does a 50-sample
insert inside a transaction (ROLLBACK) to validate, then the full bulk commit.
"""
from __future__ import annotations
import argparse, csv, json, uuid
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
GRIN_TSV = ROOT / "data/bronze/grin/extracted/NameUsage.tsv"
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"


def load_grin_species() -> list[dict]:
    out = []
    with GRIN_TSV.open(encoding="utf-8", errors="replace", newline="") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            if (row.get("col:rank") or "").lower() != "species":
                continue
            if (row.get("col:status") or "").strip():
                continue
            name = (row.get("col:scientificName") or "").strip()
            if not name:
                continue
            out.append({
                "grin_id": (row.get("col:ID") or "").strip(),
                "scientific_name": name,
                "authorship": (row.get("col:authorship") or "").strip(),
            })
    return out


def load_grin_genus_family() -> dict[str, str]:
    """genus_name -> family_name, walking the GRIN parent chain up to family."""
    rows = []
    with GRIN_TSV.open(encoding="utf-8", errors="replace", newline="") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            rows.append((row.get("col:ID"), (row.get("col:rank") or "").lower(),
                         (row.get("col:parentID") or "").strip(),
                         (row.get("col:scientificName") or "").strip()))
    by_id = {rid: (rank, parent, name) for rid, rank, parent, name in rows}
    g2f = {}
    for rid, rank, parent, name in rows:
        if rank != "genus":
            continue
        cur = parent
        depth = 0
        fam_name = None
        while cur and cur in by_id and depth < 15:
            cr, cp, cn = by_id[cur]
            if cr == "family":
                fam_name = cn
                break
            cur = cp
            depth += 1
        if fam_name:
            g2f[name.lower()] = fam_name
    return g2f


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true", help="counts only (default)")
    ap.add_argument("--apply", action="store_true", help="insert after 50-sample pre-check")
    args = ap.parse_args()
    if not GRIN_TSV.exists():
        print("missing", GRIN_TSV); return 2

    grin = load_grin_species()
    g2f = load_grin_genus_family()
    print(f"GRIN accepted species: {len(grin)}; GRIN genus->family map: {len(g2f)}")

    con = duckdb.connect(str(DB))
    wh_names = {n for (n,) in con.execute(
        "SELECT lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL").fetchall()}
    wh_gen = {n: i for i, n in con.execute("SELECT lower(name), id FROM genera").fetchall()}
    wh_fam = {n: i for i, n in con.execute("SELECT lower(name), id FROM families").fetchall()}

    new = [g for g in grin if g["scientific_name"].lower() not in wh_names]
    print(f"Already in warehouse: {len(grin) - len(new)}")
    print(f"NEW (not in warehouse): {len(new)}")

    resolvable = 0
    gen_in_wh = 0
    fam_in_wh = 0
    need_gen_insert = 0
    need_fam_insert = 0
    no_family = 0
    sample_new = []
    seen_gen, seen_fam = set(), set()
    for g in new:
        genus = g["scientific_name"].split()[0].lower()
        fam = g2f.get(genus)
        if fam is None:
            no_family += 1
            continue
        resolvable += 1
        if genus in wh_gen:
            gen_in_wh += 1
        elif genus not in seen_gen:
            seen_gen.add(genus); need_gen_insert += 1
        if fam.lower() in wh_fam:
            fam_in_wh += 1
        elif fam.lower() not in seen_fam:
            seen_fam.add(fam.lower()); need_fam_insert += 1
        if len(sample_new) < 50:
            sample_new.append(g)

    print(f"  resolvable (genus->family found in GRIN): {resolvable}")
    print(f"  UNRESOLVABLE (no GRIN family):           {no_family}")
    print(f"  genus already in warehouse:              {gen_in_wh}")
    print(f"  genus to insert:                         {need_gen_insert}")
    print(f"  family already in warehouse:             {fam_in_wh}")
    print(f"  family to insert:                        {need_fam_insert}")

    if not args.apply:
        con.close()
        return 0

    # ---- --apply: 50-sample pre-check inside a transaction, then ROLLBACK ----
    print("\nPRE-CHECK: inserting 50-sample in transaction (rollback)...")
    con.execute("BEGIN")
    try:
        _bulk_insert(con, sample_new, g2f, wh_gen, wh_fam)
    except Exception as e:
        con.execute("ROLLBACK")
        print("PRE-CHECK FAILED:", e); con.close(); return 1
    added = con.execute("SELECT COUNT(*) FROM species WHERE id LIKE 'grin_tmp%'").fetchone()[0]
    con.execute("ROLLBACK")
    print(f"PRE-CHECK OK: 50-sample inserted then rolled back (temp rows now {added}). Proceeding to full bulk.")

    # ---- full bulk ----
    existing_grin = {e for (e,) in con.execute(
        "SELECT external_id FROM species_identifiers WHERE source='grin'").fetchall()}
    before = con.execute("SELECT COUNT(*) FROM species").fetchone()[0]
    n_sp, n_gen, n_fam, n_id = _bulk_insert(con, new, g2f, wh_gen, wh_fam, tmp=False,
                                            existing_grin=existing_grin)
    after = con.execute("SELECT COUNT(*) FROM species").fetchone()[0]
    print(f"\nFULL BULK: species +{after-before} (inserted {n_sp}), genera +{n_gen}, families +{n_fam}, grin ids +{n_id}")
    con.execute("""COPY (SELECT * FROM species) TO ? (FORMAT PARQUET)""",
                [str((ROOT/'data/silver/species.parquet').resolve()).replace('\\','/')])
    con.execute("""COPY (SELECT * FROM genera) TO ? (FORMAT PARQUET)""",
                [str((ROOT/'data/silver/genera.parquet').resolve()).replace('\\','/')])
    con.execute("""COPY (SELECT * FROM families) TO ? (FORMAT PARQUET)""",
                [str((ROOT/'data/silver/families.parquet').resolve()).replace('\\','/')])
    con.execute("""COPY (SELECT * FROM species_identifiers) TO ? (FORMAT PARQUET)""",
                [str((ROOT/'data/silver/species_identifiers.parquet').resolve()).replace('\\','/')])
    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "GRIN Taxonomy (GBIF-hosted zip)",
        "grin_accepted_species": len(grin),
        "warehouse_before": before,
        "warehouse_after": after,
        "new_species_inserted": after - before,
        "unresolved_no_family": no_family,
        "note": "New taxa resolved genus->family from GRIN parentID; no Unknown taxa invented.",
    }
    (ROOT/'data/manifests/grin-backbone.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(json.dumps(report, indent=2))
    con.close()
    return 0


def _bulk_insert(con, items, g2f, wh_gen, wh_fam, tmp=False, existing_grin=None):
    prefix = "grin_tmp" if tmp else "grin"
    existing_grin = existing_grin or set()
    # collect needed families/genera
    need_fam = {}   # lower name -> id
    need_gen = {}   # lower name -> (id, family_id)
    sp_rows, id_rows = [], []
    for g in items:
        if g["grin_id"] in existing_grin:
            continue  # idempotent: already inserted
        name = g["scientific_name"]
        genus = name.split()[0].lower()
        fam = g2f.get(genus)
        if fam is None:
            continue
        fam_l = fam.lower()
        if fam_l in wh_fam:
            fam_id = wh_fam[fam_l]
        elif fam_l in need_fam:
            fam_id = need_fam[fam_l]
        else:
            fam_id = f"fam_{uuid.uuid4().hex[:12]}"
            need_fam[fam_l] = fam_id
        if genus in wh_gen:
            gen_id = wh_gen[genus]
        elif genus in need_gen:
            gen_id = need_gen[genus][0]
        else:
            gen_id = f"gen_{uuid.uuid4().hex[:12]}"
            need_gen[genus] = (gen_id, fam_id)
        tokens = name.split()
        epithet = tokens[1] if len(tokens) > 1 else ""
        authority = g["authorship"] or ""
        sp_id = f"{prefix}_{uuid.uuid4().hex[:12]}"
        sp_rows.append((sp_id, gen_id, epithet, authority, name, "accepted", "species"))
        id_rows.append((str(uuid.uuid4()), sp_id, "grin", g["grin_id"] or name, 1))
    # insert families
    for fl, fid in need_fam.items():
        if fl not in wh_fam:
            con.execute("INSERT INTO families VALUES (?, ?, ?)", [fid, fl, None])
    # insert genera
    for gl, (gid, fid) in need_gen.items():
        if gl not in wh_gen:
            con.execute("INSERT INTO genera VALUES (?, ?, ?, ?)", [gid, fid, gl, None])
    # insert species
    con.executemany(
        "INSERT INTO species (id, genus_id, specific_epithet, authority, scientific_name, taxonomic_status, rank) "
        "VALUES (?, ?, ?, ?, ?, ?, ?)", sp_rows)
    # insert identifiers (idempotent)
    for r in id_rows:
        con.execute(
            "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary) "
            "SELECT ?, ?, 'grin', ?, ? "
            "WHERE NOT EXISTS (SELECT 1 FROM species_identifiers WHERE source='grin' AND external_id=?)",
            [r[0], r[1], r[3], r[4], r[3]])
    return len(sp_rows), len(need_gen), len(need_fam), len(id_rows)


if __name__ == "__main__":
    raise SystemExit(main())
