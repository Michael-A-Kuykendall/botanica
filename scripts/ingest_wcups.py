#!/usr/bin/env python3
"""VINYL V2b — Ingest Kew WCUPS (World Checklist of Useful Plant Species 2020) allowlist.

Source: data/bronze/allowlist/wcups/extracted/wcups_species.tsv (from extract_wcups.py)
Rule:
  - Tag warehouse species whose scientific_name matches a WCUPS name with
    species_identifiers source='wcups' (cultivation/use signal).
  - Optionally insert WCUPS species NOT in the warehouse, but ONLY when their
    genus already exists in the warehouse (so family is known — no Unknown taxa).
    These new rows also get source='wcups'.
Idempotent: re-running skips existing wcups identifiers / species.
"""
from __future__ import annotations
import argparse, csv, json, uuid
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
NAMES_TSV = ROOT / "data/bronze/allowlist/wcups/extracted/wcups_species.tsv"
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"


def load_names() -> list[tuple[str, str, str]]:
    out = []
    with NAMES_TSV.open(encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        for row in r:
            n = (row.get("name") or "").strip()
            if n:
                out.append((n, (row.get("lsid") or "").strip(), (row.get("family") or "").strip()))
    return out


def build_genus_family(names) -> dict[str, str]:
    """genus(lower) -> family from any WCUPS row that carried a family."""
    g2f = {}
    for n, _, fam in names:
        if not fam:
            continue
        genus = n.split()[0].lower() if n.split() else ""
        if genus and genus not in g2f:
            g2f[genus] = fam
    return g2f


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", action="store_true", help="insert wcups identifiers on matching species")
    ap.add_argument("--insert-new", action="store_true", help="also insert new species for known genera")
    args = ap.parse_args()
    if not NAMES_TSV.exists():
        print("missing", NAMES_TSV); return 2

    names = load_names()
    g2f = build_genus_family(names)
    print(f"WCUPS names: {len(names)}; WCUPS genus->family map: {len(g2f)}")
    con = duckdb.connect(str(DB))
    wh = {}
    for sid, n in con.execute(
        "SELECT id, lower(trim(scientific_name)) FROM species WHERE scientific_name IS NOT NULL").fetchall():
        t = n.split()
        if len(t) >= 2:
            wh[(t[0] + " " + t[1]).lower()] = sid
    wh_gen = {n: i for i, n in con.execute("SELECT lower(name), id FROM genera").fetchall()}
    wh_fam = {n: i for i, n in con.execute("SELECT lower(name), id FROM families").fetchall()}

    def binomial(name: str) -> str:
        return " ".join(name.split()[:2]).lower()

    matched = 0
    new_candidates = 0
    seen_new_gen = set()
    for n, lsid, _ in names:
        key = binomial(n)
        if key in wh:
            matched += 1
        else:
            genus = n.split()[0].lower() if n.split() else ""
            if genus in wh_gen or genus in g2f:
                new_candidates += 1
                seen_new_gen.add(genus)

    print(f"Match existing warehouse species : {matched}")
    print(f"New (not in WH), known genus     : {new_candidates} across {len(seen_new_gen)} genera")
    print(f"New, unknown genus (skipped)     : {len(names)-matched-new_candidates}")

    if not (args.tag or args.insert_new):
        con.close()
        return 0

    # ---- apply ----
    existing = {e for (e,) in con.execute(
        "SELECT external_id FROM species_identifiers WHERE source='wcups'").fetchall()}
    tagged = 0
    if args.tag:
        for n, lsid in names:
            key = binomial(n)
            if key not in wh:
                continue
            ext = lsid or key
            if ext in existing:
                continue
            con.execute(
                "INSERT INTO species_identifiers SELECT ?, ?, 'wcups', ?, 0, current_timestamp "
                "WHERE NOT EXISTS (SELECT 1 FROM species_identifiers WHERE source='wcups' AND external_id=?)",
                [str(uuid.uuid4()), wh[key], ext, ext])
            tagged += 1
        print(f"Tagged wcups identifiers: {tagged}")

    inserted_sp = 0
    if args.insert_new:
        sp_rows, id_rows = [], []
        # track families/genera we add this run
        added_fam = {}
        added_gen = {}  # genus(lower) -> (gen_id, fam_id)
        for n, lsid, fam_pdf in names:
            key = binomial(n)
            if key in wh:
                continue
            tokens = n.split()
            if len(tokens) < 2:
                continue
            genus = tokens[0].lower()
            # resolve family: warehouse genus, or WCUPS genus->family map
            if genus in wh_gen:
                gen_id = wh_gen[genus]
                fam_id = con.execute("SELECT family_id FROM genera WHERE id=?", [gen_id]).fetchone()[0]
            else:
                fam = g2f.get(genus)
                if not fam:
                    continue
                fam_l = fam.lower()
                if fam_l in wh_fam:
                    fam_id = wh_fam[fam_l]
                elif fam_l in added_fam:
                    fam_id = added_fam[fam_l]
                else:
                    fam_id = f"fam_{uuid.uuid4().hex[:12]}"
                    added_fam[fam_l] = fam_id
                if genus in added_gen:
                    gen_id = added_gen[genus][0]
                else:
                    gen_id = f"gen_{uuid.uuid4().hex[:12]}"
                    added_gen[genus] = (gen_id, fam_id)
            epithet = tokens[1]
            authority = " ".join(tokens[2:]).strip()
            sp_id = f"wcups_{uuid.uuid4().hex[:12]}"
            sp_rows.append((sp_id, gen_id, epithet, authority, n, "accepted", "species"))
            id_rows.append((str(uuid.uuid4()), sp_id, "wcups", lsid or n))
        # insert families (no Unknown taxa)
        for fl, fid in added_fam.items():
            if fl not in wh_fam:
                con.execute("INSERT INTO families VALUES (?, ?, ?)", [fid, fl, None])
        for gl, (gid, fid) in added_gen.items():
            if gid not in wh_gen and gl not in {g.lower() for g in wh_gen}:
                con.execute("INSERT INTO genera VALUES (?, ?, ?, ?)", [gid, fid, gl, None])
        con.executemany(
            "INSERT INTO species (id, genus_id, specific_epithet, authority, scientific_name, taxonomic_status, rank) "
            "VALUES (?, ?, ?, ?, ?, ?, ?)", sp_rows)
        for r in id_rows:
            con.execute(
                "INSERT INTO species_identifiers (id, species_id, source, external_id, is_primary, created_at) "
                "SELECT ?, ?, 'wcups', ?, 0, current_timestamp "
                "WHERE NOT EXISTS (SELECT 1 FROM species_identifiers WHERE source='wcups' AND external_id=?)",
                [r[0], r[1], r[3], r[3]])
        inserted_sp = len(sp_rows)
        print(f"Inserted new warehouse species (genus->family via WCUPS): {inserted_sp}")

    # export silver
    for t in ("species", "species_identifiers"):
        p = str((ROOT / "data/silver" / f"{t}.parquet").resolve()).replace("\\", "/")
        con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "Kew World Checklist of Useful Plant Species 2020",
        "wcups_names": len(names),
        "matched_warehouse": matched,
        "tagged_identifiers": tagged,
        "inserted_new_species": inserted_sp,
        "applied_tag": bool(args.tag),
        "applied_insert_new": bool(args.insert_new),
    }
    (ROOT / "data/manifests/wcups-ingest.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
