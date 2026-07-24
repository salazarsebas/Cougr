# On-Chain Gaming: Problems Ranked by Impact, and Cougr's Response

*Part 5 of 17. The friction points a developer actually hits building an on-chain game, ranked for a Stellar/Soroban developer specifically, each paired with a concrete Cougr mitigation, existing or proposed.*

## Ranking methodology

Ranked by a combination of how often the problem blocks a developer entirely (versus merely slowing them down) and how Soroban-specific the pain is (a generic problem every platform shares is lower priority for Cougr to solve uniquely than a problem specific to Soroban's resource model).

## 1. Deciding what belongs on-chain vs. off-chain (highest impact)

This is the first design decision every team makes and the one most likely to produce an unshippable design if made wrong (either an unusably expensive contract, or a "on-chain" game that is actually centralized). Cougr's `SimpleWorld`/`ArchetypeWorld` split and its dirty-tracking/incremental persistence (`src/incremental/`) already reduce the cost of the "everything on-chain" default by only writing changed component data, but the decision itself is still undocumented as a first-class topic. **Recommendation:** a dedicated `docs/ONCHAIN_OFFCHAIN_BOUNDARY.md` (or a `PATTERNS.md` chapter) with worked examples: what Battleship keeps on-chain (board commitments) versus off-chain (ship placement negotiation), what a real-time game like Snake approximates versus what it cannot legitimately put on-chain (frame-by-frame movement) versus what it should (score, high-score commitments). This is documentation work, not engineering work, and it is the single highest-leverage document Cougr could ship next after the CLI.

## 2. Resource fees and simulation cost predictability

Soroban's resource-metered fee model (CPU instructions, storage read/write bytes, ledger entry counts) means a design that looks fine in a unit test can be prohibitively expensive or hit resource limits on-chain, and this is discovered late, often only at `stellar contract deploy`/invoke time. This is more acute on Soroban than on EVM chains because resource costs are multi-dimensional rather than a single gas number. **Recommendation:** extend `cougr_core::test::GameHarness` (already built for sandbox testing) to report simulated resource consumption per system/transition, and surface it in CI so a PR that regresses resource cost is visible before merge, not after a failed mainnet deploy. This is a natural extension of infrastructure that already exists (`benches/ecs_bench.rs`, the sandbox test harness) rather than new work from zero.

## 3. Testing and debugging on-chain game logic

Cougr is already ahead here relative to competitors: `GameHarness`, `Scenario`, `WorldFixture`, `ReplayLog`, and `SnapshotAssert` (behind the `testutils` feature) give a genuine sandbox testing story, and 3,726 lines of integration tests across every subsystem back it up. The gap is discoverability, not capability: none of this is documented as a standalone "how to test your game" guide outside of scattered usage in example test files. **Recommendation:** a `TESTING_GUIDE.md` that walks through `GameHarness` and snapshot testing as a first-class tutorial, referenced from the CLI's scaffolded project (a new project from `cougr new` should include a working test using this harness on day one, not require the developer to discover it).

## 4. Wallet UX and player onboarding

The general Web3 problem (seed phrases, wallet extensions, transaction-signing friction killing casual player conversion) is exactly what Cougr's account abstraction (`ClassicAccount`/`ContractAccount`, session keys, passkey/secp256r1 support, `authorize_with_fallback`) already targets at the contract level. The gap identified in the competitive analysis holds here too: there is no client SDK pairing this with an actual passkey registration/sign-in flow a frontend can drop in, so every team building a client has to reinvent that integration. **Recommendation:** a thin, documented TypeScript client package wrapping session creation and passkey registration against `cougr-core`'s account primitives, positioned the way Cartridge's Controller sits in front of Dojo. See [06-product-strategy.md](./06-product-strategy.md).

## 5. Indexing and client synchronization

`impl_component_observed!` already emits structured `(COUGR, set, <type>)` events specifically so external indexers can track state changes without polling full contract storage. This is good on-chain design. The off-chain half (something that actually consumes these events and exposes them as a subscribable feed or REST/GraphQL API) does not exist. **Recommendation:** rather than Cougr building and hosting an indexer itself (high cost, ongoing infra burden, premature at 8 GitHub stars), document the event schema clearly enough that existing Stellar indexing infrastructure (RPC event streaming, Horizon, or third-party indexers) can be pointed at a Cougr-based contract with a short recipe, and revisit a hosted, opinionated indexer only if usage data justifies the investment (see [07-business-strategy.md](./07-business-strategy.md)).

## 6. Multiplayer architecture and turn/state synchronization

Turn-based games (the majority of the 39 examples: Chess, Checkers, Tic-Tac-Toe, Battleship) have a relatively well-understood pattern (each move is a transaction, state advances deterministically), and Cougr's scheduler and command-queue design fit this well already. Real-time or simultaneous-action games (Snake, Pong, Asteroids) are architecturally harder on any blockchain, and Cougr does not currently document or prescribe a pattern for them (e.g., commit-reveal per tick, off-chain simulation with on-chain checkpoints). **Recommendation:** the `PATTERNS.md` expansion in item 1 should explicitly cover both categories, with the real-time category honestly framed as "approximated," not "fully on-chain," to avoid setting an expectation Soroban's throughput cannot meet.

## 7. Fairness and hidden information

This is the problem Cougr's ZK circuit suite (`hidden_cards`, `fog_of_war`, `fair_dice`, `sealed_bid`) already directly and uniquely solves among the frameworks studied. The remaining friction is the trusted-setup gap flagged in the audit: these ship with test-only proving keys, and there is no documented, accessible path for a team to run or join a production trusted-setup ceremony. **Recommendation:** this is a real production blocker for any team that wants to ship a ZK-backed game on mainnet today, and it should be treated as a roadmap item with real priority (either a coordinated multi-party ceremony the project organizes, or clear partnership guidance pointing to an existing ceremony/service), not left as a permanent asterisk. See [15-risks.md](./15-risks.md).

## 8. Discoverability (of the games themselves, and of Cougr)

Not on the original problem list as stated but observed directly in this audit: a game built with Cougr today has no natural distribution channel. There is no showcase, no "built with Cougr" registry, nothing equivalent to a game jam board. This compounds the 8-stars/51-forks traction gap: even the games that do get built are invisible. **Recommendation:** the showcase/gallery work in [08-ux-strategy.md](./08-ux-strategy.md) directly addresses this and should be treated as an ecosystem-growth problem, not a marketing afterthought.

## 9. Ecosystem fragmentation

Currently a smaller risk for Cougr than for older ecosystems simply because there is only one framework and one team, but it is worth naming as a forward risk: if Cougr succeeds, competing ECS approaches or forks could fragment the Soroban gaming developer base the way EVM gaming fragmented across Dojo/MUD/others with limited interop. **Recommendation:** the standards layer (`ops`) and observed-event schema are the natural interoperability surface; documenting them as an open convention other Soroban tools could adopt, rather than a closed Cougr-only format, protects against this earlier and more cheaply than trying to fix fragmentation after it happens. See [16-long-term-recommendations.md](./16-long-term-recommendations.md).

## 10. Monetization and retention

Genuinely out of scope for Cougr to solve directly at this stage (these are game-design and business problems, not framework problems), but the framework can avoid making them harder: the standards layer's `Pausable`/`AccessControl`/`ExecutionGuard` primitives are the right building blocks for the on-chain half of monetization (entry fees, prize pools, guild treasuries, already demonstrated in the `guild_treasury_wars` and `guild_arena` examples). No further core investment is recommended here beyond keeping those primitives well-documented as patterns rather than building anything resembling a payments product inside the ECS itself.
