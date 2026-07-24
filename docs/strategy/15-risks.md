# Risks

*Part 16 of 17. Named plainly, in the same honest-by-default spirit as the project's own `STATE_OF_REPO.md` and `PUBLIC_GAPS.md`, rather than softened.*

## Maintainer concentration risk

Git history shows two identity spellings of a single founder ("Sebastián Salazar Solano" and "Sebastián Salazar") accounting for roughly 183 of 433 commits, well over a third of all activity, with the next largest contributor at 35. Despite genuinely broad contributor participation (25+ distinct external contributors), final architectural judgment and most core-crate work currently concentrates in one person. This is normal for a project at this stage, but it is a real bus-factor risk for anything in this package that depends on sustained core-crate leadership (API stability decisions, the ZK ceremony, governance design). **Mitigation:** the Governance doc in [12-documentation-architecture.md](./12-documentation-architecture.md) should explicitly name at least one additional person with merge rights on core crate changes before the contributor base grows further, not after an availability gap forces the issue.

## ZK trusted-setup production gap

The four ZK circuit builders ship test-tier proving keys only, explicitly flagged in `internal/cougr-core-circuits/AUDIT.md` as unsafe for production. This is currently a well-documented, honestly-disclosed limitation, which is the right posture, but it becomes a real liability the moment a team ships a "privacy-preserving" game to mainnet without fully understanding the caveat, especially since the feature is one of Cougr's headline differentiators and will be marketed accordingly per [06-product-strategy.md](./06-product-strategy.md). **Mitigation:** the `cougr-zk-circuit` Skill and any CLI/doc surface touching these circuits should surface this warning unavoidably, not just in a nested internal audit file (see [11-skills-strategy.md](./11-skills-strategy.md)), and the ceremony work should be pulled forward in priority the moment a real team signals mainnet intent, ahead of its default Phase 3 sequencing if needed.

## Governance and community-conduct gap

No `CODE_OF_CONDUCT.md`, no issue/PR templates, no documented decision-making process, despite active external contribution. This is low-cost to fix (Phase 0 in the roadmap) and high-cost to leave unfixed if a conduct or contribution dispute arises with no documented process to resolve it. **Mitigation:** already sequenced first in [13-roadmap.md](./13-roadmap.md) precisely because of this asymmetry.

## Grant/sponsorship dependency

The recommended near-term business strategy in [07-business-strategy.md](./07-business-strategy.md) deliberately relies on SCF, SDF, and sponsorship funding rather than premature productization. This is the right call at this stage, but it is a real dependency on external, cyclical funding decisions outside the project's control, and it can create pressure to shape the roadmap around what a specific grant round rewards rather than what the roadmap in this package actually calls for. **Mitigation:** treat this package's roadmap as the source of truth across grant cycles; adapt framing and sequencing within a cycle's requirements, not the underlying priorities.

## Scope creep, this package's own risk

A research prompt this expansive (visual editor, marketplace, certifications, hosted playground, analytics dashboard) creates real pressure to build broadly rather than sequentially, especially once resources exist to build more than one thing at once. This package explicitly pushes back on that instinct throughout (see [05-ecosystem-vision.md](./05-ecosystem-vision.md) Tier 3, [07-business-strategy.md](./07-business-strategy.md)), but the risk is worth naming directly: the biggest way this strategy could fail is not picking the wrong priorities, it is abandoning the sequencing discipline under pressure to look more ambitious sooner. **Mitigation:** the exit criteria stated in [13-roadmap.md](./13-roadmap.md) for each phase should be treated as hard gates, not aspirational targets, before Tier 3/Phase 4 work begins.

## Competitive and ecosystem-fragmentation risk

Dojo and MUD have multi-year head starts, larger existing developer populations on their respective chains, and (in Lattice's case) venture funding Cougr does not have. Within Stellar specifically, there is currently no competing ECS framework, but that could change, and a second, incompatible approach emerging later could fragment a still-small Soroban gaming developer base before it has a chance to consolidate around one convention. **Mitigation:** per [04-onchain-gaming-research.md](./04-onchain-gaming-research.md), keep the standards layer and observed-event schema documented as open conventions rather than closed implementation details, so the ecosystem can consolidate around Cougr's conventions even if it doesn't consolidate around Cougr's code specifically, a softer, more durable form of winning than trying to prevent any alternative from ever emerging.

## Perception risk from acting on this package too literally

Following every recommendation in this document simultaneously, rather than the phased sequencing it specifies, would produce the opposite of the intended effect: a burst of half-finished public surfaces (a CLI with rough edges, a docs site with gaps, a showcase with broken links) shipped at once, which would damage trust more than the current, honest "quality exists, presentation is minimal" state does. **Mitigation:** each Tier A item in [14-prioritized-opportunities.md](./14-prioritized-opportunities.md) should be finished to the bar already set by the project's own `EXAMPLE_STANDARD.md` and `CONTRIBUTING.md` discipline before being made public, consistent with "nothing should feel unfinished" as a stated core principle.
