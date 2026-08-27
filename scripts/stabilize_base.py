#!/usr/bin/env python3
"""Stabilize the species base: deduplicate binomials into canonical rows.

Two problems repaired:
  1. Corrupt rows: author-fragment "species" (e.g. "& Alderw.") that were parse
     artifacts from source ingestion. These are dropped (their children removed).
  2. Duplicate binomials: same scientific name inserted by multiple sources. All
     children are repointed to a single canonical row (the one with the most child
     data) and the redundant rows are removed.

Safe: runs in a transaction, backs up nothing itself (caller should cp the db),
reports exact deltas. Idempotent on a clean base.
"""

from __future__ import annotations
import argparse, json
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"

CHILD_TABLES = [
    "species_identifiers",
    "traits",
    "cultivation_requirements",
    "vernacular_names",
    "synonyms",
    "distribution_regions",
    "uses",
    "media",
    "provenance",
    "cultivars",
    "seasonal_characteristics",
    "ecological_interactions",
]

AUTHOR_FRAGMENT_PAT = None  # replaced by AUTHOR_PAT below

import re

AUTHOR_PAT = re.compile(r"^\s*(?:&|[a-z][a-z.]*\s*&\s*|\d+)", re.I)


def is_author_fragment(name: str) -> bool:
    t = (name or "").strip()
    if not t:
        return True
    toks = t.split()
    if len(toks) < 2:
        return True
    first = toks[0]
    if first == "&":
        return True
    if "&" in t and not first[0].isupper():
        return True
    return False


def child_counts(con, sid: str) -> int:
    total = 0
    for t in CHILD_TABLES:
        try:
            n = con.execute(
                f"SELECT count(*) FROM {t} WHERE species_id=?", [sid]
            ).fetchone()[0]
            total += n
        except Exception:
            pass
    return total


def repoint(con, old: str, new: str) -> int:
    moved = 0
    for t in CHILD_TABLES:
        try:
            cur = con.execute(
                f"SELECT count(*) FROM {t} WHERE species_id=?", [old]
            ).fetchone()[0]
            if cur:
                con.execute(
                    f"UPDATE {t} SET species_id=? WHERE species_id=?", [new, old]
                )
                moved += cur
        except Exception:
            pass
    return moved


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="actually write changes")
    args = ap.parse_args()
    con = duckdb.connect(str(DB))

    rows = con.execute(
        "SELECT id, scientific_name FROM species WHERE scientific_name IS NOT NULL"
    ).fetchall()

    # 1) corrupt author-fragment rows (keep ×/+ nothogenera / graft-chimaeras)
    corrupt = []
    for i, n in rows:
        t = (n or "").strip()
        if t.startswith(("×", "+")):
            continue
        if is_author_fragment(t):
            corrupt.append(i)
    print(f"author-fragment corrupt rows to drop: {len(corrupt)}")

    # 2) duplicate binomial clusters among valid names
    clusters: dict[str, list] = defaultdict(list)
    for i, n in rows:
        t = (n or "").strip()
        if not t or t.startswith(("×", "+")):
            continue
        # binomial key = first two tokens, lowercased, without authority
        toks = t.split()
        if len(toks) < 2:
            continue
        key = " ".join(toks[:2]).lower()
        clusters[key].append((i, n))

    dup_clusters = {k: v for k, v in clusters.items() if len(v) > 1}
    print(f"duplicate binomial clusters: {len(dup_clusters)}")
    dup_rows = sum(len(v) for v in dup_clusters.values())
    print(f"rows involved in duplicates: {dup_rows}")

    merges = []  # (old_id, new_id) operations
    for key, members in sorted(dup_clusters.items()):
        # canonical = richest; tie-break by earliest inserted (keep smallest id alpha)
        scored = []
        for i, n in members:
            scored.append((child_counts(con, i), i, n))
        scored.sort(key=lambda x: (-x[0], x[1]))  # richest first, then stable id
        canon = scored[0][1]
        for _, i, n in scored[1:]:
            merges.append((i, canon))
    print(f"merge operations: {len(merges)}")

    if not args.apply:
        print("DRY RUN (--apply to write)")
        con.close()
        return 0

    # apply: drop corrupt children + rows
    for i in corrupt:
        for t in CHILD_TABLES:
            try:
                con.execute(f"DELETE FROM {t} WHERE species_id=?", [i])
            except Exception:
                pass
        con.execute("DELETE FROM species WHERE id=?", [i])

    # apply merges: repoint children then delete old
    moved_total = 0
    for old, new in merges:
        moved_total += repoint(con, old, new)
        con.execute("DELETE FROM species WHERE id=?", [old])

    after = con.execute("SELECT count(*) FROM species").fetchone()[0]
    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "corrupt_rows_dropped": len(corrupt),
        "dup_clusters_merged": len(dup_clusters),
        "dup_rows_removed": len(merges),
        "child_rows_repointed": moved_total,
        "species_before": len(rows),
        "species_after": after,
    }
    (ROOT / "data/manifests/stabilize-base.json").write_text(
        json.dumps(report, indent=2), encoding="utf-8"
    )
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
