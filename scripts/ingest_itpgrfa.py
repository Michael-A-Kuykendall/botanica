#!/usr/bin/env python3
"""VINYL V2b — Ingest ITPGRFA Annex I (International Treaty on PGRFA) allowlist.

Source: FAO Annex I (https://www.fao.org/plant-treaty/areas-of-work/the-multilateral-system/annex1/en)
Rule (crop-based / gene-pool approach): a warehouse species is in Annex I scope if its
GENUS is in the Annex I food-crop genus set (modulo exclusions) OR it is one of the
explicitly listed forage species. We tag such species with species_identifiers source='itpgrfa'
(an independent cultivation/use signal in build_curation.py). Tag-only (no new species):
Annex I crops are already represented in the warehouse.

Idempotent: re-running skips existing itpgrfa identifiers.
"""
from __future__ import annotations
import argparse, json, uuid
from datetime import datetime, timezone
from pathlib import Path
import duckdb

ROOT = Path(__file__).resolve().parents[1]
DB = ROOT / "data/botanica-cultivated-v0.1.duckdb"

# Annex I food-crop genera (with explicit exclusions noted in comments)
FOOD_GENERA = {
    "artocarpus", "asparagus", "avena", "beta", "brassica", "armoracia", "barbarea",
    "camelina", "crambe", "diplotaxis", "eruca", "isatis", "lepidium", "raphanobrassica",
    "raphanus", "rorippa", "sinapis", "cajanus", "cicer", "citrus", "poncirus", "fortunella",
    "cocos", "colocasia", "xanthosoma", "daucus", "dioscorea", "eleusine", "fragaria",
    "helianthus", "hordeum", "ipomoea", "lathyrus", "lens", "malus", "manihot", "musa",
    "oryza", "pennisetum", "phaseolus", "pisum", "secale", "solanum", "sorghum",
    "triticosecale", "triticum", "agropyron", "elymus", "vicia", "vigna", "zea",
}
# explicit species-level exclusions
FOOD_EXCLUDE = {
    "lepidium meyenii", "musa textilis", "phaseolus polyanthus", "solanum phureja",
    "zea perennis", "zea diploperennis", "zea luxurians",
}
# listed forage species (genus, set of epithets)
FORAGE = {
    "astragalus": {"chinensis", "cicer", "arenarius"},
    "canavalia": {"ensiformis"},
    "coronilla": {"varia"},
    "hedysarum": {"coronarium"},
    "lathyrus": {"cicera", "ciliolatus", "hirsutus", "ochrus", "odoratus", "sativus"},
    "lespedeza": {"cuneata", "striata", "stipulacea"},
    "lotus": {"corniculatus", "subbiflorus", "uliginosus"},
    "lupinus": {"albus", "angustifolius", "luteus"},
    "medicago": {"arborea", "falcata", "sativa", "scutellata", "rigidula", "truncatula"},
    "melilotus": {"albus", "officinalis"},
    "onobrychis": {"viciifolia"},
    "ornithopus": {"sativus"},
    "prosopis": {"affinis", "alba", "chilensis", "nigra", "pallida"},
    "pueraria": {"phaseoloides"},
    "trifolium": {"alexandrinum", "alpestre", "ambiguum", "angustifolium", "arvense",
                  "agrocicerum", "hybridum", "incarnatum", "pratense", "repens",
                  "resupinatum", "rueppellianum", "semipilosum", "subterraneum", "vesiculosum"},
    "andropogon": {"gayanus"},
    "agropyron": {"cristatum", "desertorum"},
    "agrostis": {"stolonifera", "tenuis"},
    "alopecurus": {"pratensis"},
    "arrhenatherum": {"elatius"},
    "dactylis": {"glomerata"},
    "festuca": {"arundinacea", "gigantea", "heterophylla", "ovina", "pratensis", "rubra"},
    "lolium": {"hybridum", "multiflorum", "perenne", "rigidum", "temulentum"},
    "phalaris": {"aquatica", "arundinacea"},
    "phleum": {"pratense"},
    "poa": {"alpina", "annua", "pratensis"},
    "tripsacum": {"laxum"},
    "atriplex": {"halimus", "nummularia"},
    "salsola": {"vermiculata"},
}


def in_scope(genus: str, epithet: str | None) -> bool:
    g = genus.lower()
    if epithet:
        binom = f"{g} {epithet.lower()}"
        if binom in FOOD_EXCLUDE:
            return False
    if g in FORAGE:
        return epithet is not None and epithet.lower() in FORAGE[g]
    if g in FOOD_GENERA:
        return binom not in FOOD_EXCLUDE if epithet else True
    return False


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="write itpgrfa identifiers")
    args = ap.parse_args()
    con = duckdb.connect(str(DB))
    rows = con.execute(
        "SELECT id, scientific_name FROM species WHERE scientific_name IS NOT NULL").fetchall()
    existing = {e for (e,) in con.execute(
        "SELECT external_id FROM species_identifiers WHERE source='itpgrfa'").fetchall()}
    matched = 0
    to_add = []
    for sid, n in rows:
        t = n.split()
        if len(t) < 2:
            continue
        if in_scope(t[0], t[1]):
            matched += 1
            key = f"{t[0].lower()} {t[1].lower()}"
            if key not in existing:
                to_add.append((sid, key))
    print(f"Warehouse species in Annex I scope: {matched}; new to tag: {len(to_add)}")
    if args.apply:
        for sid, key in to_add:
            con.execute(
                "INSERT INTO species_identifiers SELECT ?, ?, 'itpgrfa', ?, 0, current_timestamp "
                "WHERE NOT EXISTS (SELECT 1 FROM species_identifiers WHERE source='itpgrfa' AND external_id=?)",
                [str(uuid.uuid4()), sid, key, key])
        for t in ("species", "species_identifiers"):
            p = str((ROOT / "data/silver" / f"{t}.parquet").resolve()).replace("\\", "/")
            con.execute(f"COPY (SELECT * FROM {t}) TO '{p}' (FORMAT PARQUET)")
        print(f"Tagged itpgrfa identifiers: {len(to_add)}")
    report = {
        "built_at": datetime.now(timezone.utc).isoformat(),
        "source": "FAO ITPGRFA Annex I (International Treaty on PGRFA)",
        "url": "https://www.fao.org/plant-treaty/areas-of-work/the-multilateral-system/annex1/en",
        "warehouse_in_scope": matched,
        "tagged_new": len(to_add) if args.apply else 0,
        "applied": bool(args.apply),
    }
    (ROOT / "data/manifests/itpgrfa-ingest.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    con.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
