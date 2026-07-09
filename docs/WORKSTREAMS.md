# Release train workstreams (tracker of record)

**Queue:** `bd ready` · **Master epic:** `bot-az9`  
**Product:** Best free cultivated/agricultural botanical records → GitHub columnar silver + MANIFEST.

| Tracker | Use |
|---------|-----|
| **Beads (`bd`)** | Authoritative cross-session queue, deps, acceptance |
| Internal todos | Only for single-session micro-steps while executing a bead |

---

## Knock-out order

```text
B1 (KEEP filter) ─┬─► A1–A5 packaging ─► A4 Release (data-v0.2 floor)
                  │
                  └─► B2 merge harden ─► B3 GRIN + B4 FAOSTAT ─► B6 quality KEEP
                                              │
                                              └─► C1–C3 depth on KEEP only
```

### Progress log

| 2026-07-09 | A4,C1,B* | KEEP~18.5k; tag+Release data-v0.2.0; Wikidata hardiness~545; WS-A/B done

| Date | Closed | Note |
|------|--------|------|
| 2026-07-09 | B1, A1 | KEEP=3020 (~19 MB `silver_keep/`); full silver ~61 MB; both GitHub-safe |

---

## WS-A Packaging (`bot-az9.1`) — share with the world

| ID | Bead | Work |
|----|------|------|
| A1 | `bot-az9.1.1` | Silver+MANIFEST size check for GitHub |
| A2 | `bot-az9.1.2` | README DuckDB `read_parquet` load recipe |
| A3 | `bot-az9.1.3` | RELEASE_PROCESS / data-tag procedure |
| A4 | `bot-az9.1.4` | GitHub Release zip silver+MANIFEST |
| A5 | `bot-az9.1.5` | README/ROADMAP truth (cultivated product) |
| A6 | `bot-az9.1.6` | Optional CI smoke |

## WS-B Membership (`bot-az9.2`) — what belongs

| ID | Bead | Work |
|----|------|------|
| B1 | `bot-az9.2.1` | Negative filter KEEP = traits ∨ cult.req ∨ uses **(start here)** |
| B2 | `bot-az9.2.2` | Harden merge keys (ID → binomial → quarantine) |
| B3 | `bot-az9.2.3` | GRIN-Global free allowlist |
| B4 | `bot-az9.2.4` | FAOSTAT crop list |
| B5 | `bot-az9.2.5` | Kew Useful / Mansfeld if free redistributable |
| B6 | `bot-az9.2.6` | Rebuild KEEP + quality on KEEP only |

## WS-C Depth (`bot-az9.3`) — usable common concerns

| ID | Bead | Work |
|----|------|------|
| C1 | `bot-az9.3.1` | Hardiness + sunlight free sources on KEEP |
| C2 | `bot-az9.3.2` | Further enrich KEEP only (no wild bulk) |
| C3 | `bot-az9.3.3` | Uses fill if free license |
| C4 | `bot-az9.3.4` | Cultivars deferred v1.1 (chore) |

## Release floors

| Tag | Minimum beads closed |
|-----|----------------------|
| **data-v0.2** | B1 + A1–A5 (+ A4 Release) |
| **data-v0.3** | + B2–B4 + B6 |
| **data-v1.0-ish** | + C1 (or ceiling) + honest quality on KEEP |

Older R5/R6 beads (`bot-b35`, `bot-0q9`) still open — treat as parallel OSS/version cut; do not invent a third system.
