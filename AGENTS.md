# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Progress (not opinion)

- **Ruler:** [`docs/PRODUCT_ROUNDS.md`](docs/PRODUCT_ROUNDS.md) — rounds R0–R7 with measurable exits
- **Queue:** `bd ready` / `bd show <id>`
- Before work, state: **Round N, step Rx.y — exit is …**
- New feeds: always fail-fast **debug → smoke → full** (`docs/SCRAPE_FAIL_FAST.md`)
- Version cut only in **Round 6** (one coordinated data tag + crate bump)
- Do not freestyle sources outside `docs/EXECUTION_PLAN.md` / PRODUCT_ROUNDS without updating those docs

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

