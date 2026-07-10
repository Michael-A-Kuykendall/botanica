# Contributing to Botanica

## Open source, not open contribution

Botanica is **open source** but **not open contribution**.

- The code and cultivated knowledge seed are available under **MIT OR Apache-2.0** (see licenses)
- You may fork, study, use, and redistribute under those licenses
- **Pull requests are not accepted by default**
- Roadmap, schema, data membership (KEEP set), and merges are sole-maintainer decisions

This matches infrastructure projects that stay coherent under one owner (same model as SQLite-style stewardship and other Michael A. Kuykendall repos such as CrabCamera / Auxide).

## Sole developer

**Michael A. Kuykendall** is the only developer and final arbiter for this repository. There is no community merge path and no expectation of multi-maintainer governance.

## How to propose work

If you believe a change is worth discussing:

1. **Email first**: [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)
2. Describe background and the specific proposal
3. If aligned, a scoped collaboration may be arranged privately
4. Only after that will a PR be considered

**Unsolicited PRs will be closed without merge.** That is policy, not personal.

## What is welcome without a PR

| Channel | Welcome |
|---------|---------|
| **GitHub Issues** | Bug reports with repro steps; data-quality notes on the KEEP seed |
| **Email** | Security reports; collaboration proposals |
| **Forks** | Always free under the license |

## What stays maintainer-only

- Schema and API design
- Source selection, scrape pipelines, KEEP membership rules
- Public data releases and version tags
- Dependency and architecture changes
- Brand / positioning

## Code style (if invited)

- Rust 2021; `cargo fmt` and `cargo clippy`
- Tests for behavior changes
- Data changes must update MANIFEST / quality reports where relevant
- Never ship L3 personal inventory rows in the public seed

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Sponsorship

Botanica stays free. Optional support: [SPONSORS.md](SPONSORS.md) and GitHub Sponsors.
