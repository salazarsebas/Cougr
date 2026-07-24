# Cougr Ecosystem Strategy

Research and strategy package for evolving Cougr from an ECS library into a complete on-chain game development ecosystem for Stellar. Produced 2026-07-23 via a direct audit of the repository, crates.io, docs.rs, and GitHub, combined with market and competitive research. No production code was written or implemented as part of this package, per its own scope constraints; every recommendation here is a decision and its rationale, ready to be turned into implementation work separately.

## How to read this

Start with the [Executive Summary](./00-executive-summary.md) for the core finding and priority stack. If you only read one more document, read [14-prioritized-opportunities.md](./14-prioritized-opportunities.md) for what to actually do next, in order.

## Contents

| # | Document | Covers |
|---|---|---|
| 00 | [Executive Summary](./00-executive-summary.md) | The core finding and priority stack |
| 01 | [Current State Assessment](./01-current-state-assessment.md) | Architecture, DX, docs, examples, branding, governance, traction signal, technical debt, all grounded in a direct repo audit |
| 02 | [Market Research](./02-market-research.md) | Patterns from Unity, Godot, Bevy, Dojo, MUD, Cartridge, Lattice, Supabase, Vercel, shadcn/ui, GitHub, Linear, Figma, Turborepo |
| 03 | [Competitive Analysis](./03-competitive-analysis.md) | Positioning map, benchmarking scorecard, head-to-head vs. Dojo/MUD/raw soroban-sdk |
| 04 | [On-Chain Gaming Research](./04-onchain-gaming-research.md) | The biggest developer friction points, ranked by impact, each with a concrete Cougr response |
| 05 | [Ecosystem Vision](./05-ecosystem-vision.md) | The full component catalog, organized into tiers by sequencing dependency |
| 06 | [Product Strategy](./06-product-strategy.md) | Concrete specification for the CLI, client SDK, and showcase |
| 07 | [Business Strategy](./07-business-strategy.md) | Grants and sponsorship now, optional hosted services later, marketplace deferred |
| 08 | [UX Strategy](./08-ux-strategy.md) | The full "idea to production" developer journey, stage by stage, plus audience-specific notes |
| 09 | [Design Strategy](./09-design-strategy.md) | Brand, color, type, tokens, voice, and where design work applies first |
| 10 | [Repository Strategy](./10-repository-strategy.md) | Stay monorepo for Rust; split docs/site and the JS SDK into their own repos, and why |
| 11 | [Skills Strategy](./11-skills-strategy.md) | A skills.sh-compatible catalog, thin wrappers over the CLI, independently installable |
| 12 | [Documentation Architecture](./12-documentation-architecture.md) | Every document the ecosystem needs, what exists today, what's a gap, and the site IA |
| 13 | [Roadmap](./13-roadmap.md) | Five phases, gated on exit criteria and evidence, not calendar dates |
| 14 | [Prioritized Opportunities](./14-prioritized-opportunities.md) | Every opportunity in this package, ranked by impact-to-effort with reasoning shown |
| 15 | [Risks](./15-risks.md) | Maintainer concentration, ZK trusted-setup gap, governance gap, grant dependency, scope creep, fragmentation, and premature-shipping risk |
| 16 | [Long-Term Recommendations](./16-long-term-recommendations.md) | The synthesis, what should never change, what should always be re-evaluated |

## The finding in one paragraph

Cougr (`cougr-core` 1.1.0, MIT, published on crates.io, live on docs.rs) is a genuinely differentiated no_std ECS for Soroban, the only framework studied that bundles ZK-backed hidden-information circuits, passkey-based account abstraction, and an OpenZeppelin-style standards layer in one crate, backed by 39 example games and an unusually rigorous internal documentation culture (ADRs, a maturity model, honest self-audit docs). Against that substance, the repository has 8 GitHub stars and 51 forks, no CLI, no project generator, no public website, and minimal governance scaffolding, a pattern consistent with a bounty/contributor-program-driven repo whose technical quality has outpaced its discovery and onboarding funnel. The highest-leverage next moves are not the most ambitious ones: a CLI, a documentation site, a public showcase, and basic governance hygiene, in that order, before any marketplace, visual editor, or hosted infrastructure is considered.
