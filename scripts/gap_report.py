#!/usr/bin/env python3
"""Gap report: which priority lists are missing from KEEP (iterative fill).

Usage:
  python scripts/gap_report.py
  python scripts/gap_report.py --list data/lookups/priority_houseplants.txt
"""

from __future__ import annotations

import argparse
from pathlib import Path

import duckdb

ROOT = Path(__file__).resolve().parents[1]

# Starter houseplant / common indoor genera & species (editable)
DEFAULT_HOUSEPLANTS = [
    "Epipremnum aureum",
    "Monstera deliciosa",
    "Monstera adansonii",
    "Ficus elastica",
    "Ficus lyrata",
    "Ficus benjamina",
    "Spathiphyllum wallisii",
    "Dracaena fragrans",
    "Dracaena marginata",
    "Zamioculcas zamiifolia",
    "Chlorophytum comosum",
    "Aloe vera",
    "Crassula ovata",
    "Sansevieria trifasciata",
    "Dracaena trifasciata",
    "Philodendron hederaceum",
    "Philodendron bipinnatifidum",
    "Calathea ornata",
    "Goeppertia ornata",
    "Peperomia obtusifolia",
    "Begonia rex",
    "Saintpaulia ionantha",
    "Phalaenopsis amabilis",
    "Hoya carnosa",
    "Pilea peperomioides",
    "Maranta leuconeura",
    "Schefflera arboricola",
    "Hedera helix",
    "Tradescantia zebrina",
    "Aspidistra elatior",
    "Nephrolepis exaltata",
    "Codiaeum variegatum",
    "Dieffenbachia seguine",
    "Aglaonema commutatum",
    "Anthurium andraeanum",
    "Caladium bicolor",
    "Syngonium podophyllum",
    "Scindapsus pictus",
    "Rhaphidophora tetrasperma",
    "Alocasia macrorrhizos",
    "Colocasia esculenta",
    "Strelitzia reginae",
    "Yucca gigantea",
    "Beaucarnea recurvata",
    "Euphorbia milii",
    "Kalanchoe blossfeldiana",
    "Schlumbergera truncata",
    "Opuntia ficus-indica",
    "Citrus limon",
    "Ocimum basilicum",
    "Mentha spicata",
    "Rosmarinus officinalis",
    "Salvia rosmarinus",
    "Lavandula angustifolia",
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", type=Path, default=None)
    ap.add_argument(
        "--keep",
        type=Path,
        default=ROOT / "data/silver_keep/species" / "*.parquet",
    )
    args = ap.parse_args()
    names = list(DEFAULT_HOUSEPLANTS)
    if args.list and args.list.exists():
        names = [
            ln.strip()
            for ln in args.list.read_text(encoding="utf-8").splitlines()
            if ln.strip() and not ln.startswith("#")
        ]

    con = duckdb.connect()
    keep_path = args.keep.as_posix()
    present, missing = [], []
    for n in names:
        # exact or prefix (subspecies)
        hit = con.execute(
            f"""
            SELECT scientific_name FROM read_parquet('{keep_path}')
            WHERE lower(scientific_name) = lower(?)
               OR lower(scientific_name) LIKE lower(?) || ' %'
               OR lower(scientific_name) LIKE lower(?) || '%'
            LIMIT 1
            """,
            [n, n, n],
        ).fetchone()
        if hit:
            present.append((n, hit[0]))
        else:
            missing.append(n)

    print(f"priority list n={len(names)}")
    print(f"  IN keep:  {len(present)} ({100 * len(present) / len(names):.0f}%)")
    print(f"  MISSING:  {len(missing)} ({100 * len(missing) / len(names):.0f}%)")
    print("\n-- present --")
    for want, got in present[:30]:
        print(f"  OK  {want}  →  {got}")
    print("\n-- missing (scrape targets) --")
    for m in missing:
        print(f"  GAP {m}")

    out = ROOT / "data/manifests/gap-houseplants.txt"
    out.write_text("\n".join(missing) + ("\n" if missing else ""), encoding="utf-8")
    print(f"\nwrote missing list → {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
