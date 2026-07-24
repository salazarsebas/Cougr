# Prioritized Opportunities

*Part 15 of 17. Every opportunity named across this package, ranked by impact-to-effort ratio, with the reasoning made explicit so the ranking can be challenged rather than taken on faith.*

## Tier A: do next, high impact and low-to-moderate effort

1. **`cougr-cli` (`new`, `add`, `check`, `doctor`).** Highest impact of anything in this package: it is the precondition for nearly every UX-stage improvement in [08-ux-strategy.md](./08-ux-strategy.md) and the one gap every direct competitor (Dojo, MUD) closed early. Effort is moderate (a new Rust binary crate, largely templating existing canonical examples), not a research problem.
2. **Documentation site + Getting Started tutorial + on-chain/off-chain boundary guide.** Content mostly exists (`docs/` is already strong); the work is information architecture, a tutorial that doesn't exist yet, and a static site build. Named as the second-highest priority throughout this package because it is the second half of the "can a stranger start" answer, the CLI gets them a running project, the docs get them to understanding.
3. **Showcase / example gallery.** All underlying content (39 games) already exists. This is close to pure packaging effort, a data schema and a static generator, for a large perceived-quality gain.
4. **Code of Conduct, issue/PR templates, security contact.** Near-zero effort, addresses a live governance gap given 25+ active contributors. No reason to sequence this behind anything else.

## Tier B: do after Tier A lands, moderate impact, moderate effort

5. **Design and branding system.** Meaningfully raises perceived quality of the docs site and showcase the moment they exist; low value applied to the current bare README/badge setup alone, so it is sequenced to land alongside Tier A item 2/3, not before.
6. **TypeScript client SDK.** Closes a real, named competitive gap (session/passkey UX parity with Cartridge/MUD), but only matters once there are developers past onboarding who are trying to ship a client, which depends on Tier A landing first.
7. **First wave of Skills (`cougr-init`, `cougr-component`, `cougr-check`).** Genuinely cheap once the CLI exists (they are thin wrappers by design, per [11-skills-strategy.md](./11-skills-strategy.md)), but valueless before it does, hence sequenced strictly after item 1.
8. **Governance doc and RFC process.** Moderate effort (requires real deliberation on decision rights, not just writing), rising in urgency as contributor count grows, but not as time-sensitive as the Code of Conduct.

## Tier C: real but not urgent, needs evidence before committing effort

9. **ZK trusted-setup ceremony.** High impact for any team wanting production ZK circuits, but high effort and coordination cost; correctly gated on Phase 3 in [13-roadmap.md](./13-roadmap.md) rather than pulled forward, since committing scarce engineering time to a ceremony before there is a mainnet team actually blocked by it would be solving a hypothetical ahead of a real one.
10. **Hosted indexer.** Real friction point identified in [04-onchain-gaming-research.md](./04-onchain-gaming-research.md), but building and operating hosted infrastructure is expensive and ongoing; correctly gated on demonstrated repeated need, not built speculatively.
11. **RFG board and formalized bounty program.** Genuinely useful once there is a clear roadmap to point contributor energy at (which this package now provides), but a process investment more than an engineering one, and lower urgency than closing the onboarding funnel that determines whether new contributors show up at all.

## Tier D: explicitly deferred, do not start without new evidence

12. **Marketplace, package/asset registry, hosted playground, analytics dashboard, enterprise/consulting revenue lines, visual editor.** All covered in [05-ecosystem-vision.md](./05-ecosystem-vision.md) and [07-business-strategy.md](./07-business-strategy.md) with explicit gating criteria. Listed here only to be explicit that they were considered and deliberately not prioritized, not overlooked. Revisit each independently against its own stated trigger, not as a bundle, and not on a fixed timeline.

## How to use this ranking

Tiers reflect sequencing dependencies as much as raw importance, several Tier B and C items are rated as genuinely valuable but are blocked, technically or in terms of expected ROI, on a Tier A item landing first. If resourcing allows true parallel work, the design system (Tier B) can start alongside the CLI and docs site (Tier A) since it is a dependency of their launch quality, not a strict follow-on. Everything else should respect the ordering, building Tier B or C items before Tier A is complete is the most likely way to reproduce the exact problem this research diagnosed: real quality with no way for anyone to find or start using it.
