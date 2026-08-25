# Roadmap

Cougr's roadmap is phased by evidence and dependency, not fixed calendar dates. The trigger to advance is the stated exit criteria for each phase, not a target date.

This document is a living summary. The full planning rationale lives in [docs/strategy/13-roadmap.md](docs/strategy/13-roadmap.md) and is updated as assumptions are validated or invalidated.

*Last updated: 2026-Q3*

---

## Phase 0 — Governance hygiene ✅

**Status:** complete

Add the contributor-facing scaffolding a project with an active external contributor base needs: `CODE_OF_CONDUCT.md`, GitHub issue and PR templates, a public `ROADMAP.md`, and a dedicated security disclosure contact in `SECURITY.md`.

**Exit criteria met:** files exist and are linked from the README.

---

## Phase 1 — Onboarding funnel

**Status:** in progress · rough horizon: ~3 months of focused work

| Work item | Notes |
|---|---|
| `cougr-cli` (`new`, `add`, `check`, `doctor`) | New workspace member; four starter templates already exist |
| Documentation site | mdBook or equivalent in a new `cougr-site` repo; migrates existing `docs/` content |
| Showcase / example gallery | Data-driven off the existing example catalog |
| Design system | Palette, type, and tokens applied to docs site and showcase |
| SCF / SDF grant application | Scoped to Phase 1 deliverables |

**Exit criteria:** a developer with Rust installed can go from the docs site to a passing test in under two minutes; the showcase is live; GitHub star count and `cargo install cougr-cli` download counts are being tracked.

---

## Phase 2 — Ecosystem depth

**Status:** planned · gated on Phase 1 · rough horizon: 3 to 9 months out

| Work item | Notes |
|---|---|
| TypeScript client SDK (`cougr-sdk-js`) | Session, account, and event-subscription wrapper |
| First-wave Skills | `cougr-init`, `cougr-component`, `cougr-check`, `cougr-example-audit` |
| RFC process and first Governance doc | Decision-rights design deferred from Phase 0 by design |
| RFG (Request for Games) board | Directs existing bounty and contributor energy at ecosystem-shaping gaps |
| Resource-cost reporting in `GameHarness` | Addresses Soroban-specific friction identified in research |

**Exit criteria:** at least a handful of teams outside the core contributor group have shipped a testnet game using the CLI and client SDK without direct maintainer hand-holding.

---

## Phase 3 — Trust and production readiness

**Status:** planned · gated on real mainnet interest · rough horizon: 9 to 18 months out

| Work item | Notes |
|---|---|
| ZK trusted-setup resolution | Run or credibly partner into a production ceremony for the four circuit builders |
| "Cougr Verified" badge and checklist | Formalized verification process for production-grade games |
| Hosted indexer service evaluation | Triggered by multiple teams independently rebuilding the same indexing glue, not a target date |

**Exit criteria:** at least one team ships a real mainnet game with real players using Cougr, and the trusted-setup gap is resolved for any team that needs a production-grade ZK circuit.

---

## Phase 4 — Platform infrastructure

**Status:** planned · gated on sustained usage · 18+ months out, evidence-triggered

Package/asset registry, in-browser playground, analytics dashboard, and any hosted or paid services beyond the indexer. Marketplace and enterprise business lines revisited only once Phase 2/3 evidence shows independent teams producing distributable value.

---

## Metrics tracked across all phases

These are read together as a funnel — visibility → trial → contribution → production — not as independent vanity metrics.

| Metric | Baseline (2026-Q3) |
|---|---|
| GitHub stars | 8 |
| GitHub forks | 51 |
| External contributors (historical) | 25+ |
| `crates.io` downloads | tracked |
| `docs.rs` coverage | 48% |
| `cougr-cli` installs | tracking starts Phase 1 |
| Time-to-first-passing-test | measured in Phase 1 user testing |

The specific imbalance to watch: 51 forks to 8 stars is unusually high. Normalization toward parity is a signal the onboarding funnel is working.
