#!/usr/bin/env python3
"""Resolve genus → family via USDA PlantProfile (one sample symbol per genus).

Reads bronze catalog from PlantSearch, writes data/lookups/genus_family_usda.csv
"""
from __future__ import annotations

import argparse
import asyncio
import csv
import re
import sys
from pathlib import Path
from typing import Dict, Optional, Tuple

import aiohttp

API = "https://plantsservices.sc.egov.usda.gov/api/PlantProfile?symbol={symbol}"
UA = "BotanicaSeed/0.3 (research; cultivated knowledge base)"


def strip_html(s: str) -> str:
    return re.sub(r"<[^>]+>", "", s or "").strip()


def load_catalog(path: Path) -> Dict[str, str]:
    """genus -> sample USDA symbol (Species rank only)."""
    import json

    with path.open(encoding="utf-8") as f:
        rows = json.load(f)
    genus_symbol: Dict[str, str] = {}
    for row in rows:
        pl = row.get("Plant") or {}
        if pl.get("Rank") != "Species":
            continue
        symbol = pl.get("Symbol") or ""
        name = strip_html(pl.get("ScientificName") or "")
        parts = name.split()
        if not symbol or len(parts) < 2:
            continue
        genus = parts[0]
        genus_symbol.setdefault(genus, symbol)
    return genus_symbol


def family_from_profile(profile: dict) -> Optional[str]:
    for anc in profile.get("Ancestors") or []:
        if (anc.get("Rank") or "").lower() == "family":
            return strip_html(anc.get("ScientificName") or anc.get("Name") or "")
    return None


async def fetch_family(
    session: aiohttp.ClientSession, sem: asyncio.Semaphore, symbol: str
) -> Optional[str]:
    url = API.format(symbol=symbol)
    async with sem:
        try:
            async with session.get(url, timeout=aiohttp.ClientTimeout(total=40)) as resp:
                if resp.status != 200:
                    return None
                data = await resp.json(content_type=None)
                if not isinstance(data, dict):
                    return None
                return family_from_profile(data)
        except Exception:
            return None


async def resolve_all(
    genus_symbol: Dict[str, str], concurrency: int, existing: Dict[str, str]
) -> Dict[str, str]:
    out = dict(existing)
    pending = {g: s for g, s in genus_symbol.items() if g not in out}
    print(f"genera total={len(genus_symbol)} known={len(out)} to_fetch={len(pending)}", flush=True)
    if not pending:
        return out

    sem = asyncio.Semaphore(concurrency)
    headers = {"User-Agent": UA, "Accept": "application/json", "Referer": "https://plants.usda.gov/"}
    done = 0
    async with aiohttp.ClientSession(headers=headers) as session:
        items = list(pending.items())
        # chunk to print progress
        batch = 50
        for i in range(0, len(items), batch):
            chunk = items[i : i + batch]
            results = await asyncio.gather(
                *[fetch_family(session, sem, sym) for _, sym in chunk]
            )
            for (genus, _sym), fam in zip(chunk, results):
                if fam:
                    out[genus] = fam
            done += len(chunk)
            print(f"  resolved {done}/{len(pending)} map_size={len(out)}", flush=True)
            await asyncio.sleep(0.05)
    return out


def write_csv(path: Path, mapping: Dict[str, str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["genus", "family"])
        for g in sorted(mapping.keys()):
            w.writerow([g, mapping[g]])


def load_existing_csv(path: Path) -> Dict[str, str]:
    if not path.exists():
        return {}
    out: Dict[str, str] = {}
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f):
            g, fam = row.get("genus"), row.get("family")
            if g and fam:
                out[g] = fam
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--catalog",
        default="data/bronze/usda_catalog/plant_search_pct.json",
    )
    ap.add_argument("--out", default="data/lookups/genus_family_usda.csv")
    ap.add_argument("--seed-lookup", default="data/lookups/genus_family.csv")
    ap.add_argument("--concurrency", type=int, default=12)
    args = ap.parse_args()

    catalog = Path(args.catalog)
    if not catalog.exists():
        print("missing catalog", catalog, file=sys.stderr)
        return 2

    genus_symbol = load_catalog(catalog)
    existing = load_existing_csv(Path(args.seed_lookup))
    existing.update(load_existing_csv(Path(args.out)))

    mapping = asyncio.run(resolve_all(genus_symbol, args.concurrency, existing))
    write_csv(Path(args.out), mapping)
    print(f"wrote {args.out} entries={len(mapping)}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
