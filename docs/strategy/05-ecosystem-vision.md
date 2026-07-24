# Ecosystem Vision

*Part 6 of 17. The long-term shape of the Cougr ecosystem, organized into tiers by sequencing, not by importance. Every component is designed to integrate naturally with the others while remaining independently usable, per the composability principle stated in the project's own north star.*

## Design rule for every component below

Before any component is added to this vision it has to answer one question: does the ecosystem materially fail to function without it, or is it additive. Anything additive is sequenced after the components that are load-bearing. This is the discipline that keeps "definitive ecosystem" from collapsing into "build everything," which is the most common failure mode of ambitious platform strategies, and the one this research is explicitly asked to challenge rather than default into.

## Tier 0: exists today, needs no new engineering

**ECS core (`cougr-core`).** Stable foundation, already differentiated (privacy, account abstraction, standards). No action needed here beyond continuing the existing maturity-tiered release discipline.

**Examples.** 39 games, real content. The gap is presentation (Tier 1), not quantity.

**Internal documentation.** ADRs, maturity model, changelog, self-audit docs. The gap is public surfacing (Tier 1), not authorship.

## Tier 1: the onboarding funnel (near-term, highest leverage)

These four components together convert "technically excellent, invisible" into "technically excellent, discoverable and easy to start with." They are sequenced first because every later component (showcase, learning platform, dashboards) depends on people actually getting through onboarding first.

- **CLI and project generator** (`cougr` binary): `cougr new`, `cougr add <component>`, `cougr check`. Detailed in [06-product-strategy.md](./06-product-strategy.md).
- **Documentation site**: an mdBook build of the existing `docs/` content plus a real Getting Started tutorial, hosted on GitHub Pages. Detailed in [12-documentation-architecture.md](./12-documentation-architecture.md).
- **Game showcase / example gallery**: a static site (can be the same mdBook/docs site or a lightweight companion) presenting the 39 examples with screenshots, category tags, and maturity badges, linking back to source and, where deployed, a testnet contract explorer link.
- **Design and branding system**: a small, documented set of colors, type, and spacing rules applied consistently across the logo, README, docs site, and showcase. Detailed in [09-design-strategy.md](./09-design-strategy.md).

## Tier 2: ecosystem growth (medium-term, gated on Tier 1 landing)

These make sense once there is a real front door for people to walk through. Building them before Tier 1 would be decorating a house with no doors.

- **Client SDK (TypeScript)**: wraps session/account-abstraction primitives and observed-event consumption for frontend developers, closing the gap identified against Dojo/Cartridge and MUD in [03-competitive-analysis.md](./03-competitive-analysis.md).
- **Skills catalog** (skills.sh-compatible): `cougr-init`, `cougr-component`, `cougr-example-audit`, `cougr-zk-circuit`, and others. Detailed in [11-skills-strategy.md](./11-skills-strategy.md). These are a distribution multiplier for the CLI and examples, not a separate product.
- **Learning platform / tutorial track**: a structured, multi-step tutorial series (not just reference docs) building one real game from scratch across several lessons. This is where "designed for educators and hackathon participants" is actually delivered, rather than asserted.
- **Request for Games (RFG) board**: a maintained list of game categories or mechanics the project would like to see built (a mirror of the existing 39-example catalog's gaps), used to direct bounty/contributor-program energy (which the fork/star ratio shows is already active) toward ecosystem-shaping work instead of arbitrary tasks.
- **Verification / production-readiness badge**: a lightweight, documented checklist (extending the existing `EXAMPLE_STANDARD.md` and `MATURITY_MODEL.md`) that lets a game or contract be marked "Cougr Verified," meaning it passes the standard test/build/hygiene bar. This is cheap to build (it is largely already implemented as `scripts/enforce_hygiene.sh` logic) and gives the showcase a credible quality signal.

## Tier 3: platform infrastructure (long-term, gated on evidence of real usage)

These are the components most strategy documents jump to first, and the ones this research explicitly recommends deferring, because building them without usage data is building for a demand that has not been demonstrated yet.

- **Package/asset registry beyond crates.io**: a curated registry of community-built components, systems, and ZK circuits that plug into `cougr-core`. Justified once there are multiple teams building reusable pieces worth sharing outside the core examples; premature today with a single core team producing nearly all content.
- **Hosted indexer service**: a managed version of the event-consumption recipe described in [04-onchain-gaming-research.md](./04-onchain-gaming-research.md). Justified once enough live games exist that developers are independently building and re-building the same indexing glue.
- **In-browser playground**: compiling and running a Cougr contract sandbox entirely client-side (WASM in WASM). Technically appealing but expensive to build well and easy to build badly (a broken or slow playground actively damages trust more than no playground). Sequenced after the CLI has proven the onboarding pattern works, as an accelerant, not a replacement.
- **Analytics / developer dashboard**: usage metrics across the ecosystem (downloads, active contracts, showcase traffic). Only meaningful once there is enough volume for the numbers to say anything; a dashboard with three data points is a liability, not an asset.
- **Marketplace and monetization surfaces**: explicitly the last tier. See [07-business-strategy.md](./07-business-strategy.md) for why introducing a marketplace before there is a real user base risks signaling extraction before the ecosystem has demonstrated it can generate value in the first place.
- **Visual editor**: deliberately excluded from this roadmap entirely, not merely deferred. Rendering and scene editing are solved problems owned by other tools; Cougr's job is to be an excellent state and logic layer those tools can target, not to compete with Unity or Godot on their own ground. Revisit only if a specific, concrete integration (e.g., a Godot or Unity plugin that talks to `cougr-core` contracts) is requested by real users, at which point it is an integration, not a from-scratch visual editor.

## Composability across tiers

Every Tier 1 and Tier 2 component is designed to be independently useful: a developer can use the CLI without the docs site, read the docs site without installing the CLI, browse the showcase without building anything, and adopt the client SDK without touching sessions at all. The connective tissue is data, not code coupling, the CLI scaffolds projects that the docs site's tutorials reference, the showcase pulls its content from the same `EXAMPLE_STANDARD.md`-governed example directory the CLI's `cougr add` command reads from, and the Skills catalog is a thin distribution wrapper around the CLI rather than a parallel implementation. This mirrors the pattern observed in Supabase and Vercel in [02-market-research.md](./02-market-research.md): one authoritative core, several independently adoptable surfaces around it, no component requiring another to provide value.
