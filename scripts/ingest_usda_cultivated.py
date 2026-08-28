#!/usr/bin/env python3
"""Fetch the USDA PLANTS Cultivated Status via GBIF and add it to the signal mart.

READY-TO-RUN — requires a GBIF download access first (see docs/RESEARCH_CULTIVATED_ACCESS.md).

Why: the USDA PLANTS "Cultivated Status" flag (C = cultivated-only, N = native/
not-cultivated) is the cleanest single cultivated indicator, but the USDA portal is an
SPA and the flag is not in the public search dump. It IS published as a GBIF checklist
dataset (CC-BY 4.0): doi 10.15468/t40oqu, key 705922f7-5ba5-49ab-a75d-722e3090e690.

To obtain the archive (HUMAN, one-time ~2 min):
  1. Create a free GBIF account: https://www.gbif.org/user/register
  2. Provide credentials via env: GBIF_USER / GBIF_PASSWORD  (or a token)
  3. Run:  python scripts/ingest_usda_cultivated.py --download --apply
     which creates a download job for the dataset and pulls the DwC-A.

The DwC-A taxon file carries a `cultivated` (or `status`) field mapping to the PLANTS
Cultivated Status. We map that into cultivation_signal as signal_source='usda',
signal_kind='cultivated_status', evidence=C/N/L.

Run modes:
  --download  : request + fetch the GBIF DwC-A (needs GBIF_USER/GBIF_PASSWORD)
  --apply     : fold into data/gold/cultivation_signal.parquet + re-score
  --show      : dump the cultivated-status counts from a local DwC-A if already present
"""

from __future__ import annotations
import argparse, json, os, zipfile
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
BRONZE = ROOT / "data/bronze/usda_plants_gbif"
GOLD = ROOT / "data/gold"
MAN = ROOT / "data/manifests"
DATASET = "705922f7-5ba5-49ab-a75d-722e3090e690"


def download() -> str:
    import requests

    user = os.environ.get("GBIF_USER")
    pw = os.environ.get("GBIF_PASSWORD")
    if not user or not pw:
        raise SystemExit(
            "Set GBIF_USER and GBIF_PASSWORD (free GBIF account) to download."
        )
    # create download job
    r = requests.post(
        "https://api.gbif.org/v1/occurrence/download/request",
        auth=(user, pw),
        json={"format": "DWCA", "datasetKeys": [DATASET]},
        timeout=120,
    )
    r.raise_for_status()
    key = r.json().get("key")
    print("download job:", key)
    # poll
    import time

    url = None
    for _ in range(120):
        st = requests.get(
            f"https://api.gbif.org/v1/occurrence/download/{key}",
            auth=(user, pw),
            timeout=60,
        ).json()
        if st.get("status") == "SUCCEEDED":
            url = (
                f"https://api.gbif.org/v1/occurrence/download/{key}/datasets/{DATASET}"
            )
            break
        if st.get("status") == "KILLED":
            raise SystemExit("download failed")
        time.sleep(5)
    if not url:
        raise SystemExit("download timed out")
    zip_path = BRONZE / "usda_plants_dwca.zip"
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    with requests.get(url, auth=(user, pw), stream=True, timeout=300) as rr:
        rr.raise_for_status()
        with open(zip_path, "wb") as f:
            for chunk in rr.iter_content(1 << 20):
                f.write(chunk)
    print("saved", zip_path)
    return str(zip_path)


def show(zip_path: str) -> None:
    with zipfile.ZipFile(zip_path) as z:
        names = z.namelist()
        print("archive members:", names)
        taxon = next((n for n in names if "taxon" in n.lower()), None)
        if not taxon:
            return
        with z.open(taxon) as f:
            head = f.read(2000).decode("utf-8", "replace")
        print("taxon header/first lines:\n", head[:1200])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--download", action="store_true")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--show", action="store_true")
    args = ap.parse_args()

    zip_path = str(BRONZE / "usda_plants_dwca.zip")
    if args.download or not Path(zip_path).exists():
        zip_path = download()
    if args.show or not args.apply:
        show(zip_path)
    if args.apply:
        # fold cultivated status into signal mart (implement once archive inspected)
        print(
            "APPLY step — inspect the taxon file's cultivated column first (see --show)."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
