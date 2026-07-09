#!/usr/bin/env python3
"""GBIF vernacular fail-fast scrape: debug → smoke → full.

Flow per scientific name:
  1) GET /v1/species/match?name=
  2) GET /v1/species/{usageKey}/vernacularNames (paginated)

Only stores vernacular names (no occurrences).
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import requests

UA = "Botanica/0.3 (OSS cultivated knowledge; github.com/Michael-A-Kuykendall/botanica)"
BASE = "https://api.gbif.org/v1"
SESSION = requests.Session()
SESSION.headers.update({"User-Agent": UA, "Accept": "application/json"})

DEFAULT_DEBUG = [
    "Quercus alba",
    "Aloe vera",
    "Camellia sinensis",
    "Ginkgo biloba",
    "Mentha spicata",
]


def ts() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")


def match_species(name: str) -> dict[str, Any] | None:
    r = SESSION.get(
        f"{BASE}/species/match",
        params={"name": name, "strict": "false"},
        timeout=45,
    )
    r.raise_for_status()
    j = r.json()
    if j.get("matchType") == "NONE" or not j.get("usageKey"):
        return None
    return j


def fetch_vernaculars(usage_key: int) -> list[dict]:
    out: list[dict] = []
    offset = 0
    while True:
        r = SESSION.get(
            f"{BASE}/species/{usage_key}/vernacularNames",
            params={"limit": 100, "offset": offset},
            timeout=45,
        )
        r.raise_for_status()
        j = r.json()
        results = j.get("results") or []
        out.extend(results)
        if j.get("endOfRecords", True) or not results:
            break
        offset += len(results)
        if offset > 500:
            break
    return out


def normalize(name: str, match: dict | None, verns: list[dict]) -> dict:
    if not match:
        return {
            "query_name": name,
            "matched": False,
            "gbif_key": None,
            "canonical_name": None,
            "vernacular_names": [],
            "source": "GBIF",
            "license": "CC BY 4.0",
        }
    names = []
    seen = set()
    for v in verns:
        vn = (v.get("vernacularName") or "").strip()
        lang = (v.get("language") or "und").strip() or "und"
        if not vn:
            continue
        key = (vn.lower(), lang.lower())
        if key in seen:
            continue
        seen.add(key)
        names.append(
            {
                "name": vn,
                "language": lang,
                "preferred": bool(v.get("preferred") or v.get("isPreferredName")),
                "source_dataset": v.get("source"),
            }
        )
    return {
        "query_name": name,
        "matched": True,
        "gbif_key": match.get("usageKey") or match.get("speciesKey"),
        "canonical_name": match.get("canonicalName") or match.get("scientificName"),
        "status": match.get("status"),
        "confidence": match.get("confidence"),
        "vernacular_names": names,
        "has_en": any(
            (n.get("language") or "").lower() in ("en", "eng", "en-us", "en-gb")
            for n in names
        ),
        "source": "GBIF",
        "license": "CC BY 4.0",
    }


def scrape_one(name: str, sleep: float) -> tuple[dict, dict]:
    try:
        match = match_species(name)
        if sleep:
            time.sleep(sleep)
        verns: list[dict] = []
        if match and match.get("usageKey"):
            verns = fetch_vernaculars(int(match["usageKey"]))
            if sleep:
                time.sleep(sleep)
        raw = {"query": name, "match": match, "vernaculars": verns}
        return raw, normalize(name, match, verns)
    except Exception as e:
        return {"query": name, "error": str(e)}, {
            "query_name": name,
            "matched": False,
            "error": str(e),
            "vernacular_names": [],
            "has_en": False,
        }


def coverage(norms: list[dict]) -> dict:
    n = len(norms) or 1
    matched = sum(1 for x in norms if x.get("matched"))
    any_v = sum(1 for x in norms if x.get("vernacular_names"))
    en = sum(1 for x in norms if x.get("has_en"))
    return {
        "n": len(norms),
        "pct_matched": round(100 * matched / n, 2),
        "pct_any_vernacular": round(100 * any_v / n, 2),
        "pct_en_vernacular": round(100 * en / n, 2),
        "gate_pass_en_ge_40_of_batch": (100 * en / n) >= 40.0,
    }


def load_names(path: Path | None, limit: int | None) -> list[str]:
    if path is None:
        return list(DEFAULT_DEBUG)
    names: list[str] = []
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix.lower() == ".csv":
        import csv
        from io import StringIO

        r = csv.DictReader(StringIO(text))
        key = None
        if r.fieldnames:
            for cand in ("scientific_name", "name"):
                if cand in r.fieldnames:
                    key = cand
                    break
            key = key or r.fieldnames[0]
        for row in r:
            n = (row.get(key) or "").strip()
            if n:
                names.append(n)
    else:
        for line in text.splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                names.append(line.split("\t")[-1].strip())
    if limit:
        names = names[:limit]
    return names


def write_artifacts(out: Path, raws: list, norms: list, cov: dict) -> None:
    stamp = ts()
    for sub in ("raw", "normalized", "reports"):
        (out / sub).mkdir(parents=True, exist_ok=True)
    (out / "raw" / f"GBIF_raw_{stamp}.json").write_text(
        json.dumps(raws, indent=2), encoding="utf-8"
    )
    (out / "normalized" / f"GBIF_norm_{stamp}.json").write_text(
        json.dumps(norms, indent=2), encoding="utf-8"
    )
    (out / "reports" / f"coverage_{stamp}.json").write_text(
        json.dumps(cov, indent=2), encoding="utf-8"
    )
    print(json.dumps(cov, indent=2))
    print(f"wrote under {out}")


def run_batch(names: list[str], rate: float, workers: int) -> tuple[list, list]:
    raws: list = []
    norms: list = []
    if workers <= 1:
        for i, name in enumerate(names, 1):
            raw, norm = scrape_one(name, rate)
            raws.append(raw)
            norms.append(norm)
            if i % 50 == 0 or i == len(names):
                print(
                    f"  {i}/{len(names)} en={sum(1 for n in norms if n.get('has_en'))}",
                    flush=True,
                )
        return raws, norms

    done = 0
    with ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(scrape_one, n, rate): n for n in names}
        for fut in as_completed(futs):
            raw, norm = fut.result()
            raws.append(raw)
            norms.append(norm)
            done += 1
            if done % 100 == 0 or done == len(names):
                print(
                    f"  {done}/{len(names)} en={sum(1 for n in norms if n.get('has_en'))}",
                    flush=True,
                )
    return raws, norms


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", choices=["debug", "smoke", "sample", "full"], default="debug")
    ap.add_argument("--names-file", type=Path, default=None)
    ap.add_argument("--smoke-count", type=int, default=5)
    ap.add_argument("--sample-count", type=int, default=1000)
    ap.add_argument("--output", type=Path, default=Path("data/bronze/gbif_vern"))
    ap.add_argument("--rate", type=float, default=0.05)
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--skip-fail-fast", action="store_true")
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1]
    out = args.output if args.output.is_absolute() else root / args.output
    names_file = args.names_file
    if names_file and not names_file.is_absolute():
        names_file = root / names_file

    if args.phase == "debug":
        names = DEFAULT_DEBUG
        print("FAIL-FAST debug", names, flush=True)
        raws, norms = run_batch(names, args.rate, 1)
        for n in norms:
            print(n.get("query_name"), "matched", n.get("matched"), "en", n.get("has_en"), "n", len(n.get("vernacular_names") or []))
        cov = coverage(norms)
        write_artifacts(out / "debug", raws, norms, cov)
        return 0 if cov["pct_matched"] >= 60 else 1

    if args.phase == "smoke":
        names = load_names(names_file, args.smoke_count) if names_file else DEFAULT_DEBUG[: args.smoke_count]
        print(f"FAIL-FAST smoke n={len(names)}", flush=True)
        raws, norms = run_batch(names, args.rate, min(4, args.workers))
        cov = coverage(norms)
        write_artifacts(out / "smoke", raws, norms, cov)
        return 0 if cov["pct_matched"] >= 50 else 1

    if args.phase == "sample":
        if not names_file:
            print("sample needs --names-file", file=sys.stderr)
            return 2
        names = load_names(names_file, args.sample_count)
        print(f"GBIF sample n={len(names)}", flush=True)
        raws, norms = run_batch(names, args.rate, args.workers)
        cov = coverage(norms)
        write_artifacts(out / "sample", raws, norms, cov)
        return 0

    if not names_file:
        print("full needs --names-file", file=sys.stderr)
        return 2
    names = load_names(names_file, None)
    if not args.skip_fail_fast:
        pre = names[: args.smoke_count]
        print(f"pre-flight smoke n={len(pre)}", flush=True)
        raws_p, norms_p = run_batch(pre, args.rate, min(4, args.workers))
        cov_p = coverage(norms_p)
        if cov_p["pct_matched"] < 50:
            write_artifacts(out / "preflight_fail", raws_p, norms_p, cov_p)
            return 1
    print(f"GBIF full n={len(names)}", flush=True)
    raws, norms = run_batch(names, args.rate, args.workers)
    cov = coverage(norms)
    write_artifacts(out / "full", raws, norms, cov)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
