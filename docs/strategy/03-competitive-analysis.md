# Competitive Analysis and Benchmarking Scorecard

*Part 4 of 17. Direct comparison against the closest analogues, plus a scored benchmark across the categories the research prompt specified.*

## Positioning map

Cougr's real competitive set is narrower than "every game engine." Rendering and client-side engines (Unity, Unreal, Godot) are not competitors, they are potential *clients* sitting on top of a Cougr-powered contract, since Cougr has no rendering layer and should not build one. The genuine competitive set is on-chain state and logic frameworks: **Dojo** (Starknet), **MUD** (EVM/OP Stack), and, more loosely, general Soroban contract development without any ECS framework at all (raw `soroban-sdk`), which is the actual default choice a Stellar developer makes today in the absence of Cougr.

Within that set, Cougr's differentiation is real and specific: it is the only one of the three with built-in ZK-backed hidden-information primitives and passkey-based account abstraction shipped in the same crate as the ECS itself. Dojo and MUD both have session/account-abstraction stories, but as ecosystem-level infrastructure (Cartridge Controller, MUD's own patterns) built alongside the framework rather than inside it. Neither has an equivalent to Cougr's four pre-built ZK game circuits (hidden cards, fog of war, fair dice, sealed bid) as a first-party, importable feature.

## Benchmarking scorecard

Scored 1 (absent or a real liability) to 5 (best-in-class, a reason developers choose the platform). Godot and Bevy included as the aspirational engine-side bar even though they are not direct on-chain competitors; Dojo and MUD are the direct competitors.

| Category | Cougr (today) | Dojo | MUD | Bevy | Godot |
|---|---|---|---|---|---|
| Developer experience (first 5 min) | 2 | 4 | 4 | 3 | 5 |
| Documentation | 4 | 4 | 3 | 4 | 5 |
| Learning curve | 2 | 3 | 3 | 2 | 4 |
| CLI / scaffolding | 1 | 5 | 5 | 2 (cargo-generate, third-party) | 5 (built in) |
| Examples | 4 (content) / 2 (presentation) | 3 | 3 | 5 | 5 |
| Templates / starters | 1 | 4 | 4 | 3 | 5 |
| Community visibility | 2 | 4 | 3 | 5 | 5 |
| Extensibility / architecture | 4 | 4 | 4 | 5 | 4 |
| Production readiness | 3 | 3 | 3 | 4 | 5 |
| Onboarding (docs to running app) | 2 | 4 | 4 | 3 | 5 |
| Branding / visual identity | 1 | 4 | 3 | 4 | 5 |
| On-chain-specific features (privacy, account abstraction) | 5 | 4 | 3 | n/a | n/a |

**Reading the table.** Cougr's only categories at parity or ahead of the direct competitors are architecture/extensibility and on-chain-specific features, exactly where the earlier audit said the real engineering investment has gone. Every category that scores low (CLI, templates, branding, community visibility, onboarding) is a packaging and distribution gap, not a technical one, and every one of them is addressable without new core engineering. This table is the quantified version of the executive summary's central claim and should be revisited quarterly as a health check (see [13-roadmap.md](./13-roadmap.md)).

## Head-to-head notes

**Cougr vs. Dojo.** Dojo's `sozo` CLI (`sozo init`, `sozo build`, `sozo migrate`) is the feature Cougr most needs to match, not because Dojo's implementation is the correct blueprint technically, but because it proves the pattern works for exactly this category of product (ECS-on-chain). Dojo also benefits from Starknet's broader existing DeFi/L2 developer population; Cougr does not have that base rate of ambient developers to draw on within Stellar today, which makes discovery and showcase work (see [08-ux-strategy.md](./08-ux-strategy.md)) relatively more important for Cougr than it was for Dojo at the same stage.

**Cougr vs. MUD.** MUD's strongest asset is its indexer/client-sync story (`@latticexyz/store-sync`), letting a frontend subscribe to on-chain state changes without hand-rolled polling. Cougr's `impl_component_observed!` macro already emits indexer-friendly events, which is the right foundation, but there is no companion client library that consumes those events the way MUD's does. This is a scoped, concrete gap, not a wholesale rebuild (see [06-product-strategy.md](./06-product-strategy.md), client SDK recommendation).

**Cougr vs. raw `soroban-sdk`.** The real, everyday competitor for a Stellar developer today is not using any ECS at all and hand-writing contract storage logic. Cougr's pitch against that default has to be measured in minutes saved and bugs avoided, concretely: how much faster is "add a new game" with `cougr new` and a component macro than writing storage keys and getters by hand. Right now that comparison cannot even be made because there is no `cougr new`. Once the CLI exists, this before/after comparison (ideally with a real timed walkthrough, not a claim) becomes the single strongest piece of marketing content the project can produce, because it demonstrates the value proposition rather than asserting it.

## Where Cougr should not compete

Visual editors (Unity/Godot-style), a general asset pipeline, physics engines, and rendering are all explicitly out of scope, not because they are unimportant to games broadly, but because they are solved problems Cougr has no comparative advantage in, and attempting them would dilute focus away from the actual moat (on-chain logic, privacy, account abstraction). Where a visual or asset layer is eventually justified, it should integrate an existing client engine or framework rather than being built from scratch inside Cougr. See [05-ecosystem-vision.md](./05-ecosystem-vision.md).
