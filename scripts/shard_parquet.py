#!/usr/bin/env python3
"""Shard (and verify) parquet tables into parts each under a target size.

Convention (see docs/DATA_PARQUET.md):
  - Every parquet table lives in its own directory:  <dir>/<table>/part-NNNNNN.parquet
  - No single part may exceed TARGET_MB (default 40) — comfortably under the 50 MB
    GitHub warning / 100 MB hard limit, so we never need Git LFS.
  - DuckDB reads a whole table via the glob:  read_parquet('<dir>/<table>/*.parquet')

Usage:
  shard   --data data/silver_keep data/silver      # split any flat/oversize tables
  verify  --data data/silver_keep --verify-only    # exit 1 if any part > target (CI)
"""

from __future__ import annotations
import argparse, math, shutil, sys
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
TARGET_MB_DEFAULT = 40  # well under the 50 MB GitHub warning / 100 MB hard limit


def _p(p: Path) -> str:
    return str(p.resolve()).replace("\\", "/")


def _flat_files(data_dir: Path) -> list[Path]:
    return sorted(p for p in data_dir.glob("*.parquet") if p.is_file())


def _table_dirs(data_dir: Path) -> list[Path]:
    return sorted(d for d in data_dir.iterdir() if d.is_dir())


def _part_mb(parts: list[Path]) -> float:
    return max((p.stat().st_size for p in parts), default=0) / 1e6


def verify(data_dir: Path, target_mb: int, out=sys.stdout) -> bool:
    """Return True if every parquet part (and flat file) is <= target_mb."""
    ok = True
    for p in _flat_files(data_dir):
        mb = p.stat().st_size / 1e6
        if mb > target_mb:
            ok = False
            print(f"  OVER {target_mb}MB ({mb:.1f}M): {p}", file=out)
    for tdir in _table_dirs(data_dir):
        parts = sorted(tdir.glob("*.parquet"))
        if not parts:
            continue
        mb = _part_mb(parts)
        if mb > target_mb:
            ok = False
            for part in parts:
                if part.stat().st_size / 1e6 > target_mb:
                    print(
                        f"  OVER {target_mb}MB ({part.stat().st_size / 1e6:.1f}M): {part}",
                        file=out,
                    )
    return ok


def _shard_flat(con, src: Path, tdir: Path, target_mb: int) -> None:
    tdir.mkdir(parents=True, exist_ok=True)
    n = con.execute(f"SELECT count(*) FROM read_parquet('{_p(src)}')").fetchall()[0][0]
    if n == 0:
        con.execute(
            f"COPY (SELECT * FROM read_parquet('{_p(src)}')) TO '{_p(tdir / 'part-000000.parquet')}' (FORMAT PARQUET)"
        )
        return
    n_bytes = src.stat().st_size
    # rows per part scaled so each part ~= target_bytes
    rows_per = max(1, math.ceil(n * (target_mb * 1e6) / max(1, n_bytes)))
    n_parts = max(1, math.ceil(n / rows_per))
    rows_per = math.ceil(n / n_parts)

    con.execute(
        f"CREATE OR REPLACE TEMP TABLE _src AS SELECT * FROM read_parquet('{_p(src)}')"
    )
    cols = [c[0] for c in con.execute("SELECT * FROM _src LIMIT 0").description]
    col_list = ", ".join(c if c.isidentifier() else f'"{c}"' for c in cols)
    for i in range(n_parts):
        off = i * rows_per
        lim = rows_per if i < n_parts - 1 else n - off
        con.execute(
            f"COPY (SELECT {col_list} FROM _src OFFSET {off} LIMIT {lim}) "
            f"TO '{_p(tdir / f'part-{i:06d}.parquet')}' (FORMAT PARQUET)"
        )
    con.execute("DROP TABLE IF EXISTS _src")


def shard(data_dir: Path, target_mb: int, con) -> None:
    for flat in _flat_files(data_dir):
        if flat.stat().st_size / 1e6 <= target_mb:
            # small flat table: still normalize into a single-part dir
            tdir = data_dir / flat.stem
            if not tdir.is_dir():
                tdir.mkdir(parents=True, exist_ok=True)
                con.execute(
                    f"COPY (SELECT * FROM read_parquet('{_p(flat)}')) TO '{_p(tdir / 'part-000000.parquet')}' (FORMAT PARQUET)"
                )
            flat.unlink()
            continue
        tdir = data_dir / flat.stem
        if tdir.exists():
            shutil.rmtree(tdir)
        _shard_flat(con, flat, tdir, target_mb)
        flat.unlink()
    # re-check oversized directories
    for tdir in _table_dirs(data_dir):
        parts = sorted(tdir.glob("*.parquet"))
        if parts and _part_mb(parts) > target_mb:
            src = data_dir / f".{tdir.name}.tmp.parquet"
            paths = ",".join(f"'{_p(p)}'" for p in parts)
            con.execute(
                f"COPY (SELECT * FROM read_parquet([{paths}])) TO '{_p(src)}' (FORMAT PARQUET)"
            )
            shutil.rmtree(tdir)
            _shard_flat(con, src, tdir, target_mb)
            src.unlink()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", nargs="+", default=["data/silver_keep", "data/silver"])
    ap.add_argument("--target-mb", type=int, default=TARGET_MB_DEFAULT)
    ap.add_argument("--verify-only", action="store_true", help="only check sizes (CI)")
    args = ap.parse_args()

    con = duckdb.connect()
    bad = False
    for d in args.data:
        data_dir = (ROOT / d) if not Path(d).is_absolute() else Path(d)
        if not data_dir.is_dir():
            continue
        if args.verify_only:
            if not verify(data_dir, args.target_mb):
                bad = True
        else:
            shard(data_dir, args.target_mb, con)
            if not verify(data_dir, args.target_mb):
                bad = True
    con.close()
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
