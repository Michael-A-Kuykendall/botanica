#!/usr/bin/env python3
"""POWO fail-fast scrape: debug → smoke → sample/full.

Public API provides: accepted name, synonyms, locations (WGSRPD), lifeform, climate.
Uses field is NOT present on api/2/taxon (2026) — documented as source ceiling.

Phases:
  debug  — 3 known taxa, print raw+norm
  smoke  — first N from list, coverage report
  sample — N names (default 1000) for gate
  full   — entire list (pre-flight smoke unless --skip-fail-fast)
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
BASE = "https://powo.science.kew.org/api/2"

DEFAULT_DEBUG = [
    "Quercus alba",
    "Aloe vera",
    "Camellia sinensis",
    "Ginkgo biloba",
    "Mentha spicata",
]


def ts() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")


def _session() -> requests.Session:
    s = requests.Session()
    s.headers.update({"User-Agent": UA, "Accept": "application/json"})
    return s


def _get(sess: requests.Session, url: str, params: dict | None = None, retries: int = 6) -> requests.Response:
    """GET with backoff on 429/5xx. POWO rate-limits concurrent scrapes hard."""
    delay = 1.5
    last: requests.Response | None = None
    for attempt in range(retries):
        last = sess.get(url, params=params, timeout=60)
        if last.status_code == 429 or last.status_code >= 500:
            time.sleep(delay)
            delay = min(delay * 1.7, 45.0)
            continue
        last.raise_for_status()
        return last
    assert last is not None
    last.raise_for_status()
    return last


def search_name(sess: requests.Session, name: str) -> dict[str, Any] | None:
    r = _get(sess, f"{BASE}/search", params={"q": name, "perPage": 5})
    results = r.json().get("results") or []
    # prefer accepted exact-ish match
    for res in results:
        if res.get("name", "").lower() == name.lower() and res.get("accepted", True):
            return res
    for res in results:
        if res.get("accepted", True):
            return res
    return results[0] if results else None


def fetch_taxon(sess: requests.Session, fq_id: str) -> dict[str, Any]:
    r = _get(sess, f"{BASE}/taxon/{fq_id}")
    return r.json()


def normalize(scientific_name: str, search: dict | None, detail: dict | None) -> dict:
    if not detail:
        return {
            "query_name": scientific_name,
            "matched": False,
            "powo_id": None,
            "accepted_name": None,
            "synonyms": [],
            "locations": [],
            "lifeform": None,
            "climate": None,
            "uses": [],  # always empty — API ceiling
            "authors": None,
            "family": None,
            "taxonomic_status": None,
        }
    syns = []
    for s in detail.get("synonyms") or []:
        if isinstance(s, dict) and s.get("name"):
            syns.append(
                {
                    "name": s.get("name"),
                    "author": s.get("author"),
                    "fqId": s.get("fqId"),
                    "rank": s.get("rank"),
                }
            )
    locs = detail.get("locations") or []
    if isinstance(locs, dict):
        locs = list(locs.keys()) if locs else []
    return {
        "query_name": scientific_name,
        "matched": True,
        "powo_id": detail.get("fqId") or (search or {}).get("fqId"),
        "accepted_name": detail.get("name"),
        "authors": detail.get("authors"),
        "family": detail.get("family"),
        "taxonomic_status": detail.get("taxonomicStatus"),
        "synonyms": syns,
        "locations": locs if isinstance(locs, list) else [],
        "lifeform": detail.get("lifeform"),
        "climate": detail.get("climate"),
        "uses": [],
        "source": "POWO",
        "license": "CC BY 4.0",
    }


def scrape_one(name: str, sleep: float, sess: requests.Session | None = None) -> tuple[dict, dict]:
    sess = sess or _session()
    search = None
    detail = None
    try:
        search = search_name(sess, name)
        if sleep:
            time.sleep(sleep)
        if search and search.get("fqId"):
            try:
                detail = fetch_taxon(sess, search["fqId"])
            except Exception as te:
                # keep search hit; partial norm from search only
                raw = {"query": name, "search": search, "taxon": None, "taxon_error": str(te)}
                partial = {
                    "query_name": name,
                    "matched": True,
                    "powo_id": search.get("fqId"),
                    "accepted_name": search.get("name"),
                    "authors": search.get("author"),
                    "family": search.get("family"),
                    "taxonomic_status": "Accepted" if search.get("accepted") else None,
                    "synonyms": [],
                    "locations": [],
                    "lifeform": None,
                    "climate": None,
                    "uses": [],
                    "source": "POWO",
                    "license": "CC BY 4.0",
                    "partial": True,
                    "error": str(te),
                }
                return raw, partial
            if sleep:
                time.sleep(sleep)
        raw = {"query": name, "search": search, "taxon": detail}
        return raw, normalize(name, search, detail)
    except Exception as e:
        return {"query": name, "search": search, "error": str(e)}, normalize(name, search, detail) | {
            "error": str(e)
        }


def coverage(norms: list[dict]) -> dict:
    n = len(norms) or 1
    matched = sum(1 for x in norms if x.get("matched"))
    with_syn = sum(1 for x in norms if x.get("synonyms"))
    with_loc = sum(1 for x in norms if x.get("locations"))
    with_life = sum(1 for x in norms if x.get("lifeform"))
    with_clim = sum(1 for x in norms if x.get("climate"))
    with_uses = sum(1 for x in norms if x.get("uses"))
    useful = sum(
        1
        for x in norms
        if x.get("matched")
        and (x.get("synonyms") or x.get("locations") or x.get("lifeform") or x.get("climate"))
    )
    return {
        "n": len(norms),
        "pct_matched": round(100 * matched / n, 2),
        "pct_useful_enrichment": round(100 * useful / n, 2),
        "pct_synonyms": round(100 * with_syn / n, 2),
        "pct_locations": round(100 * with_loc / n, 2),
        "pct_lifeform": round(100 * with_life / n, 2),
        "pct_climate": round(100 * with_clim / n, 2),
        "pct_uses": round(100 * with_uses / n, 2),
        "uses_api_ceiling": True,
        "uses_note": "POWO api/2/taxon does not expose plant uses (2026); gate uses lifeform/climate/locations/synonyms",
        "gate_pass_useful_ge_50": (100 * useful / n) >= 50.0,
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
            for cand in ("scientific_name", "name", "Scientific Name"):
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
            if not line or line.startswith("#"):
                continue
            # symbol\tname or name only
            if "\t" in line:
                parts = line.split("\t")
                names.append(parts[-1].strip())
            elif "," in line and path.suffix == ".txt":
                names.append(line.split(",")[0].strip())
            else:
                names.append(line)
    if limit:
        names = names[:limit]
    return names


def write_artifacts(out: Path, raws: list, norms: list, cov: dict) -> None:
    stamp = ts()
    (out / "raw").mkdir(parents=True, exist_ok=True)
    (out / "normalized").mkdir(parents=True, exist_ok=True)
    (out / "reports").mkdir(parents=True, exist_ok=True)
    (out / "raw" / f"POWO_raw_{stamp}.json").write_text(
        json.dumps(raws, indent=2), encoding="utf-8"
    )
    (out / "normalized" / f"POWO_norm_{stamp}.json").write_text(
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
    sleep = rate
    # Default sequential: POWO 429s aggressively under concurrency.
    if workers <= 1:
        sess = _session()
        for i, name in enumerate(names, 1):
            raw, norm = scrape_one(name, sleep, sess)
            raws.append(raw)
            norms.append(norm)
            if i % 25 == 0 or i == len(names):
                print(
                    f"  {i}/{len(names)} matched={sum(1 for n in norms if n.get('matched'))} err={sum(1 for r in raws if r.get('error'))}",
                    flush=True,
                )
        return raws, norms

    def job(name: str):
        return scrape_one(name, sleep, _session())

    done = 0
    with ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(job, n): n for n in names}
        for fut in as_completed(futs):
            raw, norm = fut.result()
            raws.append(raw)
            norms.append(norm)
            done += 1
            if done % 50 == 0 or done == len(names):
                print(
                    f"  {done}/{len(names)} matched={sum(1 for n in norms if n.get('matched'))}",
                    flush=True,
                )
    return raws, norms


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", choices=["debug", "smoke", "sample", "full"], default="debug")
    ap.add_argument("--names-file", type=Path, default=None)
    ap.add_argument("--smoke-count", type=int, default=5)
    ap.add_argument("--sample-count", type=int, default=1000)
    ap.add_argument("--output", type=Path, default=Path("data/bronze/powo"))
    ap.add_argument("--rate", type=float, default=0.55, help="sleep seconds between HTTP calls per worker")
    ap.add_argument("--workers", type=int, default=1, help="POWO: keep 1; concurrent → 429")
    ap.add_argument("--min-useful-pct", type=float, default=50.0)
    ap.add_argument("--skip-fail-fast", action="store_true")
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1]
    out = args.output if args.output.is_absolute() else root / args.output

    if args.phase == "debug":
        names = DEFAULT_DEBUG
        print("FAIL-FAST debug", names, flush=True)
        raws, norms = run_batch(names, args.rate, 1)
        for n in norms:
            print("---", n.get("query_name"), "matched=", n.get("matched"))
            print(json.dumps({k: n[k] for k in n if k != "synonyms"}, indent=2)[:800])
            print("synonyms_n=", len(n.get("synonyms") or []), "locs=", len(n.get("locations") or []))
        cov = coverage(norms)
        write_artifacts(out / "debug", raws, norms, cov)
        return 0 if cov["pct_matched"] >= 60 else 1

    names_file = args.names_file
    if names_file and not names_file.is_absolute():
        names_file = root / names_file

    if args.phase == "smoke":
        names = load_names(names_file, args.smoke_count) if names_file else DEFAULT_DEBUG[: args.smoke_count]
        print(f"FAIL-FAST smoke n={len(names)}", flush=True)
        raws, norms = run_batch(names, args.rate, min(2, args.workers))
        cov = coverage(norms)
        write_artifacts(out / "smoke", raws, norms, cov)
        ok = cov["pct_matched"] >= 60 and cov["pct_useful_enrichment"] >= args.min_useful_pct
        return 0 if ok else 1

    if args.phase == "sample":
        if not names_file:
            print("sample requires --names-file", file=sys.stderr)
            return 2
        names = load_names(names_file, args.sample_count)
        print(f"POWO sample n={len(names)}", flush=True)
        raws, norms = run_batch(names, args.rate, args.workers)
        cov = coverage(norms)
        write_artifacts(out / "sample", raws, norms, cov)
        return 0 if cov["gate_pass_useful_ge_50"] else 1

    # full
    if not names_file:
        print("full requires --names-file", file=sys.stderr)
        return 2
    names = load_names(names_file, None)
    if not args.skip_fail_fast:
        pre = names[: args.smoke_count]
        print(f"pre-flight smoke n={len(pre)}", flush=True)
        raws_p, norms_p = run_batch(pre, args.rate, min(2, args.workers))
        cov_p = coverage(norms_p)
        if cov_p["pct_matched"] < 60 or cov_p["pct_useful_enrichment"] < args.min_useful_pct:
            write_artifacts(out / "preflight_fail", raws_p, norms_p, cov_p)
            print("FAIL-FAST: abort full", file=sys.stderr)
            return 1
    print(f"POWO full n={len(names)}", flush=True)
    raws, norms = run_batch(names, args.rate, args.workers)
    cov = coverage(norms)
    write_artifacts(out / "full", raws, norms, cov)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
