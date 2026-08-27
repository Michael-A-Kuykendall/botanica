# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Progress (not opinion)

- **Ruler:** [`docs/PRODUCT_ROUNDS.md`](docs/PRODUCT_ROUNDS.md) — rounds R0–R7 with measurable exits
- **Queue:** `bd ready` / `bd show <id>`
- Before work, state: **Round N, step Rx.y — exit is …**
- New feeds: always fail-fast **debug → smoke → full** (`docs/SCRAPE_FAIL_FAST.md`)
- **Parquet:** every table ships as `<dir>/<table>/part-NNNNNN.parquet` with each part ≤ **40 MB** (under GitHub's 50 MB warning / 100 MB hard limit) — **no Git LFS**. Read via `read_parquet('<dir>/<table>/*.parquet')`. Writers MUST route output through `scripts/shard_parquet.py`; CI enforces `--verify-only`. See `docs/DATA_PARQUET.md`.
- Version cut only in **Round 6** (one coordinated data tag + crate bump)
- Do not freestyle sources outside `docs/EXECUTION_PLAN.md` / PRODUCT_ROUNDS without updating those docs

## Versioning (the rule — read before touching version numbers)

Three digits = **major.minor.patch** (left to right). A **patch** (bug fix, docs, CI fix,
minor edit) bumps the **rightmost** digit: `0.3.0 → 0.3.1`. A **minor** (feature / API or
schema addition) bumps the **middle**: `0.3.0 → 0.4.0`. Only a real, justified stable
release bumps **major** — never a CI fix, never a doc change, never "because it's
Tuesday."

Rules:
- **We stay in `0.x`.** `1.0.0` means "stable API, real users depend on it." We have
  neither, so no major bump, ever, until that is genuinely true.
- **Tag only real releases.** Each CI fix, pipeline edit, or tiny correction is a
  **patch** (`0.3.x → 0.3.(x+1)`), NOT a new tag and NOT a minor. Never tag a CI fix as
  a version.
- **One coordinated cut** at Round 6/V10: changelog entry + tag + release assets together.
- Counter-example to avoid (from ContextLite): an AI bumping `0.9-alpha1..alpha10` then
  `v1.0.x` dozens of times for every CI fix — a version-number train wreck. Do not
  reproduce it here.

Current state (record): `0.3.0` is the crate version at HEAD; last tagged release is
`v0.2.0`; `0.3.0` was bumped (minor, justified — schema 0.4 + features) but never
tagged. V10 tags `v0.3.0`; later edits go to `0.3.1`, etc.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

