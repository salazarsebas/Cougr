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

## Example Catalog

| Example | Category | Focus |
|---|---|---|
| `spawn_and_move` | **Starter / canonical** | Complete idiomatic Cougr pattern: `SorobanGame` + `impl_component_observed!` + typed ECS |
| `angry_birds` | Physics-inspired arcade | Projectile logic and destructible-state style gameplay |
| `arkanoid` | Arcade | Paddle, collision, and brick lifecycle management |
| `asteroids` | Arcade | Entity-heavy movement, collisions, and spawning |
| `battleship` | Board / hidden information | Commit-reveal and selective state disclosure |
| `bomberman` | Grid action | Tile updates, hazards, and timed interactions |
| `chess` | Board / strategy | Rule validation and proof-oriented move flow |
| `flappy_bird` | Arcade | Tight tick-loop updates and obstacle generation |
| `geometry_dash` | Reflex | Deterministic timing and obstacle progression |
| `guild_arena` | Account patterns | Social recovery and multi-device gameplay |
| `murdoku` | Puzzle | Ephemeral ECS validation and creator registry |
| `pac_man` | Maze action | Grid navigation and adversarial movement patterns |
| `pokemon_mini` | Turn-based battle | Combat sequencing and match state transitions |
| `pong` | Arcade | Minimal competitive loop and ECS fundamentals |
| `proof_of_hunt` | Hidden-state exploration | stellar-zk style proof verification and x402 premium actions |
| `rock_paper_scissors` | Commit-reveal | Hidden choices and reveal resolution |
| `snake` | Arcade | Growth mechanics and collision rules |
| `space_invaders` | Wave shooter | Formation movement and repeated tick systems |
| `tap_battle` | Casual competitive | Lightweight action resolution and progression |
| `tetris` | Puzzle | Piece state, rotation, and board clearing |
| `treasure_hunt` | Hidden-state exploration | Off-chain Merkle map commitments with on-chain proof-gated discovery and sparse fog-of-war |
| `tic_tac_toe` | Board | Small-state deterministic turn handling |
| `trading_card_game` | Card / strategy | Structured turns, card effects, and state composition |

## Choosing A Reference

Use examples by pattern, not only by genre:

| If you need to study | Good starting points |
|---|---|
| **First-time onboarding** | `spawn_and_move` |
| Basic ECS structure | `spawn_and_move`, `pong`, `snake`, `tetris` |
| Rich components (`Address`, `Vec`) | `tic_tac_toe`, `trading_card_game` |
| Hidden state or commit-reveal | `battleship`, `rock_paper_scissors` |
| Turn-based logic | `tic_tac_toe`, `pokemon_mini`, `chess` |
| Account abstraction patterns | `guild_arena` |
| Larger multi-entity loops | `asteroids`, `space_invaders`, `pac_man` |
| ZK proof / fog-of-war | `treasure_hunt`, `proof_of_hunt` |

## Preferred Runtime Shape

For new examples and new production contracts, prefer the modern Cougr runtime path:

- `GameApp` as the entrypoint
- explicit stage placement for systems
- `SimpleQueryBuilder` for non-trivial world scans
- table storage for hot-loop gameplay state

For examples that intentionally preserve older patterns, keep them explicitly
documented as compatibility or transition references rather than presenting
them as the default onboarding path.

## Canonical References

The maintained reference architectures for Cougr should be read in this order:

- `spawn_and_move`: **start here** — canonical onboarding example showing the full modern Cougr pattern (`SorobanGame`, `impl_component_observed!`, typed ECS)
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

For contribution expectations across the repository, see [CONTRIBUTING.md](../CONTRIBUTING.md).
