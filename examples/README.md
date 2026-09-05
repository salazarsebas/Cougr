# Examples

This directory contains standalone game projects built on top of `cougr-core`. The examples are intended to serve two purposes:

- demonstrate how the framework can be applied to different gameplay models
- provide reference implementations for architecture, storage, authorization, and verification patterns

The catalog is expected to grow over time. Documentation in this directory should therefore avoid hard dependencies on exact counts or one-off example narratives.

## Example Quality Standard

Every example in this directory is expected to meet a documented quality bar. See [EXAMPLE_STANDARD.md](./EXAMPLE_STANDARD.md) for the full standard, including dependency requirements, module structure, README expectations, testing categories, and the checklist used by cleanup issues.

## How To Use The Examples

Each example lives in its own directory and can be built independently. Every example must pass both of the following commands:

```bash
cd examples/<example-name>
cargo test
stellar contract build
```

`stellar contract build` is required for all examples, not optional. An example that only passes `cargo build` is not a valid Soroban contract.

## Recommended Reading Order

New contributors and external developers should read the examples in the following order to build understanding incrementally:

1. **`spawn_and_move`** - Start here. Demonstrates the complete idiomatic Cougr pattern: `SorobanGame` + `impl_component_observed!` + typed ECS.
2. **`tic_tac_toe`** - Turn-based game with `impl_rich_component!` for `Address` and `Vec` fields. Shows how to structure a small-state game.
3. **`snake`** - Arcade loop, `GameApp` tick model, and basic ECS. Introduces the stage-scheduling model for continuous gameplay.
4. **`battleship`** - Hidden-information / commit-reveal flow using `privacy::stable` Merkle primitives. Entry point for privacy-aware games.
5. **`session_arena`** - `session::SessionManager` lifecycle and multi-round state. Demonstrates the session UX pattern (Beta).
6. **`hidden_hand`** - First ZK example. `circuits::HiddenHandBuilder` for hidden-card proof flows (Experimental).
7. **`fog_explorer`** - `circuits::FogExplorerBuilder` for fog-of-war exploration with Merkle proofs (Experimental).
8. **`dice_duel`** - `circuits::FairDiceBuilder` for fair dice roll with on-chain Groth16 verification (Experimental).
9. **`blind_auction`** - `circuits::SealedBidBuilder` for sealed-bid auctions with commit-reveal ZK proofs (Experimental).
10. **`guild_arena`** - Account recovery and multi-device authorization. Demonstrates `auth` and `ops` standards.

After reading the canonical examples, explore transitional examples for additional patterns and genre-specific techniques.

## Example Catalog

### Canonical examples

These are the maintained reference architectures. They are held to the full standard in [EXAMPLE_STANDARD.md](./EXAMPLE_STANDARD.md) and stay current as `cougr-core` evolves.

| Example | Category | Focus | Preview |
|---|---|---|---|
| `spawn_and_move` | **Starter / canonical** | Complete idiomatic Cougr pattern: `SorobanGame` + `impl_component_observed!` + typed ECS | - |
| `tic_tac_toe` | **Rich components / canonical** | Turn-based game with `impl_rich_component!` for `Address` and `Vec` fields | [preview.svg](./tic_tac_toe/preview.svg) |
| `session_arena` | **Session UX / canonical** | `session::SessionManager` lifecycle and multi-round state (Beta) | - |
| `hidden_hand` | **ZK circuits / canonical** | `circuits::HiddenHandBuilder` - hidden-card ZK proof flow (Experimental) | - |
| `fog_explorer` | **ZK circuits / canonical** | `circuits::FogExplorerBuilder` - fog-of-war exploration with Merkle proofs (Experimental) | - |
| `dice_duel` | **ZK circuits / canonical** | `circuits::FairDiceBuilder` - fair dice roll with on-chain verification (Experimental) | - |
| `blind_auction` | **ZK circuits / canonical** | `circuits::SealedBidBuilder` - sealed-bid auction with commit-reveal ZK (Experimental) | - |
| `snake` | **Arcade (GameApp) / canonical** | Arcade loop, `GameApp` tick model, basic ECS | - |
| `battleship` | **Hidden information / canonical** | Commit-reveal and selective state disclosure using `privacy::stable` | [preview.svg](./battleship/preview.svg) |
| `guild_arena` | **Authentication & recovery / canonical** | Account abstraction, social recovery, multi-device authorization | - |

### Transitional examples

These examples were written before the current standard or intentionally preserve an older pattern for compatibility reference. They still pass `cargo test` and `stellar contract build` but may not follow the latest module structure or README depth.

| Example | Category | Focus | Preview |
|---|---|---|---|
| `ai_dungeon_master_arena` | ZK / x402 | Proof-backed encounters and x402 premium actions | - |
| `angry_birds` | Physics-inspired arcade | Projectile logic and destructible-state gameplay | - |
| `arkanoid` | Arcade | Paddle, collision, and brick lifecycle management | - |
| `asteroids` | Arcade | Entity-heavy movement, collisions, and spawning | - |
| `bomberman` | Grid action | Tile updates, hazards, and timed interactions | - |
| `checkers` | Board | Jump/capture rules and multi-step turn validation | [preview.svg](./checkers/preview.svg) |
| `chess` | Board / strategy | Rule validation and proof-oriented move flow | - |
| `connect_four` | Board | Gravity-drop column logic and vertical/horizontal/diagonal win detection | - |
| `cross_asset_racing_league` | Multi-asset racing | Payment-gated boost mechanics with cross-asset payment flows | - |
| `flappy_bird` | Arcade | Tight tick-loop updates and obstacle generation | - |
| `geometry_dash` | Reflex | Deterministic timing and obstacle progression | - |
| `guild_treasury_wars` | DAO / ZK | DAO-governed factions with stellar-zk commitments | - |
| `memory_match` | Card matching | Pair-reveal mechanics and memory-state tracking | - |
| `minesweeper` | Puzzle | Grid reveal, mine detection, and adjacency logic | - |
| `murdoku` | Puzzle | Ephemeral ECS validation and creator registry | - |
| `pac_man` | Maze action | Grid navigation and adversarial movement patterns | - |
| `pokemon_mini` | Turn-based battle | Combat sequencing and match state transitions | - |
| `pong` | Arcade | Minimal competitive loop and ECS fundamentals | - |
| `proof_of_hunt` | Hidden-state exploration | stellar-zk proof verification and x402 premium actions | - |
| `reversi` | Board | Piece-flipping logic and territory control | - |
| `rock_paper_scissors` | Commit-reveal | Hidden choices and reveal resolution | - |
| `shadow_draft_card_game` | Card / hidden-hand | Hidden-hand draft with SHA-256 commit-reveal | - |
| `space_invaders` | Wave shooter | Formation movement and repeated tick systems | - |
| `sudoku` | Puzzle | Grid constraints and cell-entry validation | - |
| `tap_battle` | Casual competitive | Lightweight action resolution and progression | - |
| `tetris` | Puzzle | Piece state, rotation, and board clearing | - |
| `tower_defense` | Strategy | Wave spawning, tower attacks, and health reduction | - |
| `trading_card_game` | Card / strategy | Structured turns, card effects, and state composition | - |
| `treasure_hunt` | Hidden-state exploration | Off-chain Merkle map commitments with on-chain proof-gated discovery | - |

## Choosing A Reference

Use examples by pattern, not only by genre:

| If you need to study | Good starting points |
|---|---|
| **First-time onboarding** | `spawn_and_move` |
| `SorobanGame` / `impl_soroban_game!` | `spawn_and_move`, `tic_tac_toe` |
| Basic ECS structure | `spawn_and_move`, `pong`, `snake`, `tetris` |
| Rich components (`Address`, `Vec`) | `tic_tac_toe`, `trading_card_game` |
| Session management (Beta) | `session_arena` |
| Hidden state or commit-reveal | `battleship`, `rock_paper_scissors` |
| ZK circuits (Experimental) | `hidden_hand`, `fog_explorer`, `dice_duel`, `blind_auction` |
| Arcade / GameApp tick loop | `snake`, `asteroids`, `space_invaders` |
| Turn-based logic | `tic_tac_toe`, `pokemon_mini`, `chess` |
| Account abstraction & recovery | `guild_arena` |
| Testing with `GameHarness` | `spawn_and_move`, `tic_tac_toe` (see `testutils` feature) |
| Larger multi-entity loops | `asteroids`, `space_invaders`, `pac_man` |

## Preferred Runtime Shape

For new examples and new production contracts, prefer the modern Cougr runtime path:

- `SorobanGame` + `impl_soroban_game!` for load/save boilerplate
- `GameApp` as the entrypoint for multi-system contracts
- explicit stage placement for systems
- `SimpleQueryBuilder` for non-trivial world scans
- `impl_rich_component!` for components containing `Address` or `Vec` fields
- table storage for hot-loop gameplay state

For examples that intentionally preserve older patterns, keep them explicitly documented as compatibility or transition references rather than presenting them as the default onboarding path.

## Path Dependency Note


Some canonical examples use `path = "../../"` dependencies because they exercise APIs (`circuits`, `session`, `SorobanGame`) not yet published to crates.io. These path dependencies are permitted during monorepo development but must be replaced with published crate versions before any release tag is cut. See [EXAMPLE_STANDARD.md §1.1](./EXAMPLE_STANDARD.md#11-path-dependencies-in-monorepo-development) for the full policy.

- `spawn_and_move`: **start here** - canonical onboarding example showing the full modern Cougr pattern (`SorobanGame`, `impl_component_observed!`, typed ECS)
- `tic_tac_toe`: turn-based game showing rich components (`impl_rich_component!`, `impl_soroban_game!`) for `Address` and `Vec` fields
- `session_arena`: session lifecycle, authorization scopes, and fallback direct-auth flows (`SessionManager`)
- `snake`: canonical arcade loop and `GameApp` tick model
- `battleship`: canonical hidden-information / commit-reveal flow using `privacy::stable` Merkle primitives
- `guild_arena`: canonical account recovery and multi-device authorization flow
- **ZK Circuit Reference Examples**:
  - `hidden_hand`: private card deals via `circuits::hidden_cards`
  - `fog_explorer`: private line-of-sight map verification via `circuits::fog_of_war`
  - `dice_duel`: verifiable on-chain dice rolling via `circuits::fair_dice`
  - `blind_auction`: sealed-bid auction reveals via `circuits::sealed_bid`


## Conventions

- Keep each example self-contained.
- Prefer a clear gameplay loop over framework trickery.
- Document any non-obvious contract behavior in the example's local `README.md`.
- If an example introduces a reusable pattern, reflect that pattern back into the core documentation where appropriate.

## Adding A New Example

Before adding a new example:

1. confirm the example demonstrates a pattern not already covered clearly elsewhere
2. keep the directory standalone and runnable on its own
3. include a local `README.md` with scope, commands, and design notes
4. add or update a CI workflow if the example should be validated automatically
5. classify the example as canonical or transitional and document its category

For contribution expectations across the repository, see [CONTRIBUTING.md](../CONTRIBUTING.md).
