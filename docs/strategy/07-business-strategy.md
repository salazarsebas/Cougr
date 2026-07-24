# Business Strategy

*Part 8 of 17. A sustainability model appropriate to Cougr's actual stage (pre-traction, 8 GitHub stars, no revenue, MIT-licensed), not an aspirational model borrowed from platforms that are years and significant funding ahead.*

## The honest starting point

Every "sustainable business model" section in a platform strategy document is tempting to write as if the platform already has users to monetize. Cougr does not yet. The correct business strategy for this stage is not "which revenue model," it is "how does the project fund the Tier 1 work in [05-ecosystem-vision.md](./05-ecosystem-vision.md) without compromising the openness that is core to the project's stated values, and without building monetization surfaces that signal extraction before the ecosystem has proven it creates value." Recommending a marketplace or enterprise tier right now would fail the project's own stated principle of long-term maintainability over short-term appearance of ambition.

## Near-term (0 to 12 months): grants and sponsorship, not products

Given the presence of Stellar Community Fund-oriented tooling already integrated into this project's environment (SCF-focused skills and references are already part of the working setup), the **Stellar Community Fund** is the most immediately available and appropriate funding channel, and should be treated as the primary near-term sustainability path rather than a side option. SCF funding is explicitly designed for exactly this stage of ecosystem infrastructure (open-source, pre-revenue, Stellar-native), and unlike a marketplace or hosted service, it requires no product built to receive it, only a credible plan, which this package provides the substance for.

Complementary near-term channels, in order of effort-to-value:

- **Stellar Development Foundation grants and ecosystem programs**, for the same reasons SCF applies, and because SDF has direct interest in Soroban developer tooling maturing.
- **Sponsorships from Stellar ecosystem projects** (wallets, anchors, infrastructure providers) who benefit from more games existing on Stellar, structured as no-strings sponsorship (logo placement on the docs site or showcase) rather than paid placement that could compromise the standards layer's neutrality.
- **Hackathon and bounty-program continuation**, formalized rather than incidental. The existing fork/star ratio shows this channel already works at bringing in contributors; the RFG board proposed in [05-ecosystem-vision.md](./05-ecosystem-vision.md) turns that existing energy into ecosystem-shaping output (filling example gaps, building the CLI) instead of arbitrary tasks.

None of these require compromising the MIT license or building a paid product. All are consistent with "premium quality over development speed" as a funded, deliberate effort rather than unpaid nights-and-weekends work, which is itself a maintainability risk worth naming plainly (see [15-risks.md](./15-risks.md)).

## Medium-term (12 to 24 months): optional hosted services, gated on real usage

Once Tier 1/Tier 2 components are live and there is a real base of teams building on Cougr (measured by the traction metrics in [13-roadmap.md](./13-roadmap.md), not by launch dates), a **Supabase-style split** becomes appropriate: the core (`cougr-core`, the CLI, the client SDK, all examples) stays MIT-licensed and fully usable with zero payment, forever. A hosted, optional layer, most plausibly the hosted indexer described in [04-onchain-gaming-research.md](./04-onchain-gaming-research.md), or hosted CI/testnet-provisioning for teams who want it managed, is the first legitimate revenue surface, priced modestly and explicitly framed as a convenience, not a gate. This is the same tradeoff Vercel and Supabase made, and it works because the free, self-hosted path never degrades; the paid layer only removes operational toil for teams who would rather not run their own.

## Long-term (24+ months): the pieces to defer, not the pieces to plan now

Enterprise offerings (custom SLAs, audits-as-a-service using the standards layer as a base), consulting (implementation support for studios), and educational products (a paid, structured course beyond the free tutorial track, or a certification program) are all legitimate eventual revenue lines, and all are explicitly **not** near-term priorities. Certifications in particular require an existing, credible community and a body of graduates who found value in the free learning path first, building one before that exists produces a certification nobody recognizes. The correct sequencing is: prove the free path works and grows a community, then layer revenue on top of demonstrated demand, not the reverse.

## What a marketplace requires before it makes sense

A marketplace (of components, circuits, or games) is the most frequently over-prioritized item in platform strategies of this kind, and it is explicitly sequenced to Tier 3 in the ecosystem vision for a specific reason: a marketplace requires supply (multiple independent teams producing sellable assets) and demand (buyers who trust the platform enough to pay) simultaneously, and Cougr currently has neither, the core team produces nearly all existing content. Building marketplace infrastructure now would be building a store with no vendors and no customers. The correct trigger to revisit this is evidence, not a calendar date: multiple external teams independently building and wanting to distribute Cougr-compatible components.

## Tradeoffs explicitly acknowledged

Grant funding is not guaranteed, cyclical, and can create pressure to over-index on what a specific funding round rewards rather than what developers actually need, this is a real risk and should be managed by keeping the roadmap in [13-roadmap.md](./13-roadmap.md) as the source of truth regardless of which grant cycle is currently open. Deferring monetization also means the core team's ability to work full-time on this remains dependent on grants/sponsorship rather than durable revenue for longer than a faster-monetizing strategy would require; this is accepted deliberately in exchange for protecting the open, trust-building early phase that every platform studied in [02-market-research.md](./02-market-research.md) needed to earn its later commercial success.
