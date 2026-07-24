# Roadmap

*Part 14 of 17. Phased by evidence and dependency, not fixed calendar dates, consistent with the "gated on traction" principle used throughout this package. Rough horizons given for planning purposes; the actual trigger to move to the next phase is the stated exit criteria, not the date.*

## Phase 0: Governance hygiene (do immediately, independent of everything else)

- Add `CODE_OF_CONDUCT.md`, GitHub issue templates, and a PR template.
- Publish a public `ROADMAP.md` (a living summary of this document).
- Add a dedicated security disclosure contact to `SECURITY.md`.

**Why first:** near-zero cost, addresses a present risk (25+ active contributors with no documented conduct or dispute process), and requires no design or engineering dependencies. **Exit criteria:** files exist and are linked from the README.

## Phase 1: The onboarding funnel (0 to ~3 months of focused work)

- Ship `cougr-cli` (`new`, `add`, `check`, `doctor`) as a new workspace member, per [06-product-strategy.md](./06-product-strategy.md).
- Stand up the documentation site (mdBook or equivalent) in a new `cougr-site` repository, migrating existing `docs/` content and adding the Getting Started tutorial and on-chain/off-chain boundary guide first, per [12-documentation-architecture.md](./12-documentation-architecture.md).
- Build the showcase/example gallery, data-driven off the existing example catalog.
- Apply the design system (palette, type, tokens) to the docs site and showcase, per [09-design-strategy.md](./09-design-strategy.md).
- Submit an SCF (or equivalent SDF/ecosystem) grant application scoped specifically to this phase's work, per [07-business-strategy.md](./07-business-strategy.md).

**Why this phase:** this is the entire priority stack from the executive summary; every later phase depends on people being able to discover and start with Cougr in the first place. **Exit criteria:** a developer with Rust installed can go from the docs site to a passing test in under two minutes; the showcase is live and browsable; GitHub star count and unique `cargo install cougr-cli` counts (via crates.io download stats) are being tracked as the first real traction metrics.

## Phase 2: Ecosystem depth (gated on Phase 1 landing, roughly 3 to 9 months out)

- Ship the TypeScript client SDK (session/account/event-subscription wrapper), per [06-product-strategy.md](./06-product-strategy.md), in its own `cougr-sdk-js` repository.
- Ship the first wave of Skills (`cougr-init`, `cougr-component`, `cougr-check`, `cougr-example-audit`), per [11-skills-strategy.md](./11-skills-strategy.md).
- Publish the RFC process and first Governance doc, per [12-documentation-architecture.md](./12-documentation-architecture.md).
- Launch the RFG (Request for Games) board to direct existing bounty/contributor energy at ecosystem-shaping gaps.
- Begin resource-cost reporting in `GameHarness`, addressing the Soroban-specific friction identified in [04-onchain-gaming-research.md](./04-onchain-gaming-research.md).

**Exit criteria:** at least a handful of teams outside the core contributor group have shipped a testnet game using the CLI and (where relevant) the client SDK without direct hand-holding from the maintainers; this is the evidence gate for Phase 3.

## Phase 3: Trust and production readiness (gated on real mainnet interest, roughly 9 to 18 months out)

- Resolve the ZK trusted-setup gap: either run or credibly partner into a production ceremony for the four circuit builders, per the risk flagged in [15-risks.md](./15-risks.md).
- Launch the "Cougr Verified" badge and formalize the verification checklist.
- Evaluate the hosted indexer service based on actual demand signals (multiple teams independently rebuilding the same indexing glue is the trigger, not a target date).

**Exit criteria:** at least one team ships a real, mainnet game with real players using Cougr, and the trusted-setup gap is resolved for any team that needs a production-grade ZK circuit.

## Phase 4: Platform infrastructure (gated on sustained usage, 18+ months out, evidence-triggered not calendar-triggered)

- Package/asset registry, in-browser playground, analytics dashboard, and any hosted/paid service beyond the indexer, all per the Tier 3 gating in [05-ecosystem-vision.md](./05-ecosystem-vision.md).
- Revisit marketplace and enterprise business lines only once Phase 2/3 evidence shows independent teams producing distributable value, per [07-business-strategy.md](./07-business-strategy.md).

## Metrics to track starting now, regardless of phase

GitHub stars (current baseline: 8), unique forks vs. stars ratio (current baseline: 51 forks, the specific imbalance this package is built around, watch for this normalizing toward parity as a sign the funnel is working), crates.io downloads, docs.rs coverage percentage (current baseline: 48%), number of distinct external contributors per quarter (current baseline: 25+ historically), and, once available, `cougr-cli` install counts and time-to-first-passing-test in informal user testing. None of these are vanity metrics in isolation, they are read together as a funnel: visibility (stars) to trial (CLI installs) to contribution (forks, PRs) to production (verified games).
