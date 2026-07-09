#!/usr/bin/env python3
"""Scrape USDA traits for a symbol list using botanica_usda (must run from budsy or PYTHONPATH).

Example:
  python scripts/scrape_haschar_batch.py \\
    --symbols-file data/bronze/usda_catalog/lists/haschar_all.txt \\
    --exclude-file data/bronze/usda_catalog/lists/gate3c_symbols_1000_haschar.txt \\
    --output data/bronze/haschar_full \\
    --rate 0.6 --concurrency 8
"""
from __future__ import annotations

import argparse
import asyncio
import sys
from pathlib import Path

# Allow import from sibling budsy package
ROOT = Path(__file__).resolve().parents[1]
BUDSY = ROOT.parent / "budsy"
sys.path.insert(0, str(BUDSY))

from botanica_usda.scraper import USDAPlantsScraper  # noqa: E402
from botanica_usda.mapping import normalize_usda  # noqa: E402


def load_symbols(path: Path) -> list[str]:
    return [ln.strip() for ln in path.read_text(encoding="utf-8").splitlines() if ln.strip()]


async def main_async(args) -> int:
    symbols = load_symbols(Path(args.symbols_file))
    exclude = set()
    if args.exclude_file and Path(args.exclude_file).exists():
        exclude = set(load_symbols(Path(args.exclude_file)))
    symbols = [s for s in symbols if s not in exclude]
    print(f"to_scrape={len(symbols)} excluded={len(exclude)}", flush=True)
    if not symbols:
        print("nothing to do")
        return 0

    out = Path(args.output)
    meta = {
        "symbol_source": "file",
        "symbol_file_path": str(args.symbols_file),
        "selection_method": args.selection_method,
        "excluded": len(exclude),
    }
    async with USDAPlantsScraper(
        out, rate_limit=args.rate, concurrency=args.concurrency
    ) as s:
        # chunk for progress + intermediate saves
        chunk = args.chunk
        all_raw = []
        all_norm = []
        for i in range(0, len(symbols), chunk):
            batch = symbols[i : i + chunk]
            print(f"batch {i//chunk+1}: {i+1}-{i+len(batch)} / {len(symbols)}", flush=True)
            raw = await s.fetch_many(batch)
            norm = [normalize_usda(r) for r in raw]
            all_raw.extend(raw)
            all_norm.extend(norm)
            s.save_artifacts(all_raw, all_norm, report_meta=meta)
        print("DONE", flush=True)
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--symbols-file", required=True)
    ap.add_argument("--exclude-file", default="")
    ap.add_argument("--output", default="data/bronze/haschar_full")
    ap.add_argument("--rate", type=float, default=0.6)
    ap.add_argument("--concurrency", type=int, default=8)
    ap.add_argument("--chunk", type=int, default=250)
    ap.add_argument(
        "--selection-method",
        default="haschar_all_minus_prior_gates",
    )
    return asyncio.run(main_async(ap.parse_args()))


if __name__ == "__main__":
    raise SystemExit(main())
