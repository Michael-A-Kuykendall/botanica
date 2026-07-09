#!/usr/bin/env python3
"""Probe USDA PlantProfile until we have N symbols with HasCharacteristics=true.

PlantSearch does not populate HasCharacteristics; only PlantProfile does.
"""
from __future__ import annotations

import argparse
import asyncio
import csv
import random
from pathlib import Path

import aiohttp

API = "https://plantsservices.sc.egov.usda.gov/api/PlantProfile?symbol={symbol}"
UA = "BotanicaSeed/0.3 (HasCharacteristics probe)"


async def has_char(session, sem, symbol: str) -> bool:
    async with sem:
        try:
            async with session.get(
                API.format(symbol=symbol), timeout=aiohttp.ClientTimeout(total=30)
            ) as resp:
                if resp.status != 200:
                    return False
                data = await resp.json(content_type=None)
                return bool(isinstance(data, dict) and data.get("HasCharacteristics"))
        except Exception:
            return False


async def main_async(args) -> int:
    random.seed(args.seed)
    rows = list(csv.DictReader(open(args.master, encoding="utf-8")))
    symbols = [r["symbol"] for r in rows if r.get("symbol")]
    random.shuffle(symbols)

    found: list[str] = []
    checked = 0
    sem = asyncio.Semaphore(args.concurrency)
    headers = {
        "User-Agent": UA,
        "Accept": "application/json",
        "Referer": "https://plants.usda.gov/",
    }
    async with aiohttp.ClientSession(headers=headers) as session:
        # batch probe
        i = 0
        while len(found) < args.target and i < len(symbols):
            batch = symbols[i : i + args.batch]
            i += len(batch)
            results = await asyncio.gather(*[has_char(session, sem, s) for s in batch])
            for s, ok in zip(batch, results):
                checked += 1
                if ok:
                    found.append(s)
                    if len(found) >= args.target:
                        break
            print(
                f"checked={checked} found={len(found)}/{args.target}",
                flush=True,
            )
            await asyncio.sleep(0.05)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(found[: args.target]) + "\n", encoding="utf-8")
    notes = Path(args.notes)
    notes.write_text(
        f"""# HasCharacteristics symbol list

- Target: {args.target}
- Found: {len(found)}
- Checked: {checked}
- Seed: {args.seed}
- Method: random shuffle master_species.csv, PlantProfile.HasCharacteristics==true
""",
        encoding="utf-8",
    )
    print(f"wrote {out} n={min(len(found), args.target)}")
    return 0 if found else 2


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--master", default="data/bronze/usda_catalog/master_species.csv")
    ap.add_argument(
        "--out", default="data/bronze/usda_catalog/lists/gate3c_symbols_1000_haschar.txt"
    )
    ap.add_argument(
        "--notes",
        default="data/bronze/usda_catalog/lists/gate3c_selection_notes.md",
    )
    ap.add_argument("--target", type=int, default=1000)
    ap.add_argument("--concurrency", type=int, default=12)
    ap.add_argument("--batch", type=int, default=40)
    ap.add_argument("--seed", type=int, default=20260708)
    return asyncio.run(main_async(ap.parse_args()))


if __name__ == "__main__":
    raise SystemExit(main())
