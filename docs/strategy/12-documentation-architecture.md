# Documentation Architecture

*Part 13 of 17. Every document a world-class open-source ecosystem needs, mapped against what already exists in this repository versus what is a genuine gap, organized into the information architecture the documentation site should present. This is architecture, not the documents themselves.*

## Site information architecture

A single documentation site (see [10-repository-strategy.md](./10-repository-strategy.md) for why it becomes its own repository, `cougr-site`) with six top-level sections, in this order because it mirrors the developer journey in [08-ux-strategy.md](./08-ux-strategy.md):

1. **Start** (landing page, Getting Started, Build Your First Game tutorial)
2. **Learn** (concepts, patterns, guides)
3. **Reference** (API-level docs, generated from rustdoc plus hand-written module guides)
4. **Showcase** (the example gallery)
5. **Design** (brand, UI/UX guidelines, for anyone building a client or contributing visual work)
6. **Community** (contributing, governance, security, roadmap)

## Foundation documents

| Document | Status | Notes |
|---|---|---|
| Vision / Mission | **Gap** | The README states a value proposition but there is no standalone Vision doc separating "why Cougr exists" from "how to use it." Should be short, one page, and should state the on-chain-logic-layer positioning from [00-executive-summary.md](./00-executive-summary.md) plainly. |
| Architecture | Exists (`ARCHITECTURE.md`) | Strong. Needs to be surfaced on the site's Learn section, not just linked from the README. |
| Roadmap | **Gap** | No `ROADMAP.md` exists anywhere. Should be a public, living version of [13-roadmap.md](./13-roadmap.md), updated quarterly, not a one-time publish. |
| Glossary | **Gap** | Terms like component, system, world, archetype, session key, observed component, curated surface are used precisely in the code and docs but never defined in one place for a newcomer. |
| FAQ | **Gap** | Should be sourced from actual recurring questions (Discord/issues/PR comments) once a channel exists to collect them, not invented speculatively. |

## Learning documents

| Document | Status | Notes |
|---|---|---|
| Getting Started | Partial (README quickstart) | Good as a snippet, not a full onboarding page. Should become its own page pointing at the CLI. |
| Build Your First Game (tutorial) | **Gap, highest priority** | The single most important missing document per [08-ux-strategy.md](./08-ux-strategy.md). Sequential, one game, no branching. |
| Game Patterns (by problem, not by module) | **Gap** | `PATTERNS.md` exists but is organized by API surface; needs a "by problem" companion or restructure per [08-ux-strategy.md](./08-ux-strategy.md) stage 4. |
| On-Chain / Off-Chain Boundary Guide | **Gap, second priority** | Identified as the highest-leverage missing content in [04-onchain-gaming-research.md](./04-onchain-gaming-research.md). |
| Smart Contract Patterns (Soroban-specific, not game-specific) | Partial (scattered across `PATTERNS.md`, `STANDARDS_LAYER.md`) | Should be consolidated into one Learn-section page distinguishing "Cougr patterns" from "general Soroban patterns" so readers know which parts are portable knowledge. |
| Testing Guide | Exists (`docs/learn/TESTING_GUIDE.md`) | `GameHarness`/`Scenario`/`SnapshotAssert` standalone guide covering sandbox setup, scenarios, fixtures, snapshots, replay logs, and snapshot conventions. |
| Deployment Guide | Partial (README dev commands) | Should explicitly hand off to the `stellar` CLI with a short, current recipe, consistent with the decision in [06-product-strategy.md](./06-product-strategy.md) not to wrap deployment in Cougr's own CLI. |

## Reference documents

| Document | Status | Notes |
|---|---|---|
| API reference (rustdoc) | Exists, live on docs.rs, 48% coverage | Raise coverage over time as a tracked metric (see [13-roadmap.md](./13-roadmap.md)), not a one-time push. |
| ECS Core reference | Exists (`ECS_CORE.md`) | Good. |
| Account Kernel reference | Exists (`ACCOUNT_KERNEL.md`) | Good, should gain the Beta-caveat callout treatment described in [11-skills-strategy.md](./11-skills-strategy.md). |
| Standards Layer reference | Exists (`STANDARDS_LAYER.md`) | Good, should adopt the "OpenZeppelin of Soroban gaming" framing from [06-product-strategy.md](./06-product-strategy.md). |
| Privacy Model / ZK reference | Exists (`PRIVACY_MODEL.md`) | Should prominently surface the trusted-setup production warning, not bury it in `internal/cougr-core-circuits/AUDIT.md` where a typical user won't find it. |
| Feature Flags reference | Exists (`FEATURE_FLAGS.md`) | Good. |
| Performance guide | Exists (`PERFORMANCE.md`) | Good, should link the resource-cost reporting work from [04-onchain-gaming-research.md](./04-onchain-gaming-research.md) once built. |
| Compatibility Promises / API Contract / Migration Guide | Exist | Good, keep as-is; these are exactly the kind of honest-by-default documents worth preserving as a norm. |
| CLI reference | **Gap** | Does not exist because the CLI does not exist yet; ships alongside it. |
| Client SDK reference | **Gap** | Ships alongside the SDK described in [06-product-strategy.md](./06-product-strategy.md). |

## Design documents

| Document | Status | Notes |
|---|---|---|
| Branding guide | **Gap** | See [09-design-strategy.md](./09-design-strategy.md) for full specification. |
| Color palette | **Gap** | Part of the branding guide. |
| Typography | **Gap** | Part of the branding guide. |
| Design tokens | **Gap** | Part of the branding guide. |
| UI guidelines | **Gap** | For anyone building a client (frontend) against Cougr; should draw on the `murdoku` frontend as the first worked reference. |
| UX guidelines | **Gap** | Player-facing UX guidance (wallet/session UX especially), distinct from developer UX covered in [08-ux-strategy.md](./08-ux-strategy.md). |
| Accessibility | **Gap** | Should apply to both the docs site itself and guidance for client builders; currently entirely unaddressed anywhere in the project. |

## Governance and process documents

| Document | Status | Notes |
|---|---|---|
| Contributing Guide | Exists (`CONTRIBUTING.md`) | Unusually rigorous already; keep as the model for other docs' tone. |
| Code of Conduct | **Gap, urgent** | Missing despite 25+ active external contributors. Should be added immediately, independent of any other work in this package, this is a near-zero-cost fix for a real, present governance gap. |
| Governance | **Gap** | No documented decision-making process exists. Should specify, at minimum, who can merge, how disputes over public API changes are resolved (the existing "Public API Checklist" in `CONTRIBUTING.md` is a good start but does not say who has final say), and how maintainer status is granted. |
| RFC Process | **Gap** | The ADR practice (`docs/adr/`) already covers internal architecture decisions well; an RFC process is the public-facing counterpart for changes the community should weigh in on before they happen, not just after. Recommend adopting a lightweight RFC template modeled directly on the existing ADR format, since the team already has the discipline to use it well. |
| Release Process | Exists (`RELEASE_CHECKLIST.md`, `RELEASE_STATUS.md`) | Good, keep. |
| Security Policy | Exists (`SECURITY.md`) | Good, should add a dedicated disclosure email/address, currently deferred to "maintainer channels." |
| Issue / PR templates | **Gap** | Missing at the GitHub level entirely; trivial to add and directly useful given current contributor volume. |
| Best Practices | Partial (scattered) | Should be consolidated as a single Learn-section page once the pattern-by-problem restructuring above happens, rather than maintained separately. |

## Sequencing

Urgent and near-zero-cost (do independent of everything else): Code of Conduct, issue/PR templates. Highest-leverage content work: the Getting Started/tutorial pair and the on-chain/off-chain boundary guide, both named twice already in this package because they are the two single documents most likely to change a newcomer's outcome. Everything else in this architecture should be built as the corresponding product ships (CLI reference with the CLI, SDK reference with the SDK, branding docs alongside the design system rollout), not authored speculatively ahead of the thing it documents.
