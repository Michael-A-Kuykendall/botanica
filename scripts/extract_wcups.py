#!/usr/bin/env python3
"""Extract the species list from Kew's World Checklist of Useful Plant Species (2020) PDF.

Source PDF: data/bronze/allowlist/wcups/World_Checklist_of_Useful_Plant_Species_2020.pdf
Layout per entry:
    Genus species Author
      <LSID> | <USE-CODES> | [<source indices>]

We pair each metadata line (contains '|' + LSID) with the preceding name line.
Writes:
    data/bronze/allowlist/wcups/extracted/wcups_species.tsv   (name, lsid, uses, sources)
    data/bronze/allowlist/wcups/extracted/wcups_names.txt      (one name per line)
    data/manifests/wcups-extract.json                          (counts + use histogram)
"""
from __future__ import annotations
import json, re
from pathlib import Path
from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
PDF = ROOT / "data/bronze/allowlist/wcups/World_Checklist_of_Useful_Plant_Species_2020.pdf"
OUT = ROOT / "data/bronze/allowlist/wcups/extracted"
MAN = ROOT / "data/manifests"
OUT.mkdir(parents=True, exist_ok=True)
MAN.mkdir(parents=True, exist_ok=True)

FOOTER = "World Checklist of Useful Plant Species (2020)"
META_RE = re.compile(r"^\s*\d+-\d+\s*\|")
FAM_RE = re.compile(r"^[A-Z][a-z\u00e0-\u017f]{3,}$")  # flush-left single capitalized word = family (or higher rank)


def parse():
    r = PdfReader(str(PDF))
    rows = []
    for pno, page in enumerate(r.pages, 1):
        text = page.extract_text() or ""
        # keep raw lines (indentation matters) but mark footer
        raw = text.splitlines()
        for i, ln in enumerate(raw):
            s = ln.strip()
            if not s or s.startswith(FOOTER) or s.lower().startswith("page "):
                continue
            if META_RE.match(ln):
                # name is the nearest preceding non-empty, non-footer line
                nm = None
                for j in range(i - 1, -1, -1):
                    cand = raw[j].strip()
                    if not cand or cand.startswith(FOOTER) or cand.lower().startswith("page "):
                        continue
                    nm = j
                    break
                if nm is None:
                    continue
                name = raw[nm].strip()
                # family = nearest preceding FLUSH-LEFT (no indent) single-word line,
                # scanning upward from ABOVE the name line (skip the name itself)
                family = None
                for j in range(nm - 1, -1, -1):
                    cand = raw[j]
                    if cand.startswith(" "):
                        continue  # indented genus line
                    c = cand.strip()
                    if not c or c.startswith(FOOTER) or c.lower().startswith("page "):
                        continue
                    if FAM_RE.match(c):
                        family = c
                        break
                    # else: a sibling species name (flush, multi-word) — keep scanning up
                parts = [p.strip() for p in ln.split("|")]
                lsid = parts[0]
                uses = parts[1] if len(parts) > 1 else ""
                srcs = parts[2] if len(parts) > 2 else ""
                rows.append((name, lsid, uses, srcs, family or "", pno))
    return rows

def main() -> int:
    rows = parse()
    tsv = OUT / "wcups_species.tsv"
    names = OUT / "wcups_names.txt"
    with tsv.open("w", encoding="utf-8") as f, names.open("w", encoding="utf-8") as nf:
        f.write("name\tlsid\tuses\tsources\tfamily\tpage\n")
        for name, lsid, uses, srcs, family, pno in rows:
            f.write(f"{name}\t{lsid}\t{uses}\t{srcs}\t{family}\t{pno}\n")
            nf.write(name + "\n")
    use_hist = {}
    for _, _, uses, _, _, _ in rows:
        for c in re.findall(r"[A-Z]{2}", uses):
            use_hist[c] = use_hist.get(c, 0) + 1
    fam_missing = sum(1 for r in rows if not r[4])
    report = {
        "source": "Kew World Checklist of Useful Plant Species 2020 (PDF)",
        "pdf": str(PDF),
        "total_entries": len(rows),
        "distinct_names": len({n for n, *_ in rows}),
        "entries_with_family": len(rows) - fam_missing,
        "use_code_histogram": dict(sorted(use_hist.items())),
        "note": "Two-letter use codes map to 10 EBDCS categories (ME medicines, HF human food, etc).",
    }
    (MAN / "wcups-extract.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
