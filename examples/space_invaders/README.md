# Space Invaders On-Chain Game

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

## Purpose and pattern

This example demonstrates a shooter entity-update loop on Soroban with Cougr ECS concepts. It remains transitional while the arcade examples converge on the canonical `snake` `GameApp` architecture.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | `none` | `()` | Initializes ship, invaders, score, lives, and game flags. |
| `move_ship` | `direction: i32` | `i32` | Moves the ship left/right within bounds and returns x position. |
| `shoot` | `none` | `bool` | Spawns a player bullet when possible. |
| `update_tick` | `none` | `bool` | Advances bullets, invaders, collisions, score, and game-over checks. |
| `get_score` | `none` | `u32` | Returns score. |
| `get_lives` | `none` | `u32` | Returns remaining lives. |
| `get_ship_position` | `none` | `i32` | Returns ship x position. |
| `check_game_over` | `none` | `bool` | Returns terminal state. |
| `get_active_invaders` | `none` | `u32` | Returns number of active invaders. |
| `get_entity_count` | `none` | `u32` | Returns total tracked entity count. |

## Architecture overview

```text
contract entrypoint
  ├─ reads game state from Soroban storage
  ├─ applies input or tick systems
  └─ writes updated state back to storage
```

Ship, invader, bullet, score, and lives state is currently compacted in contract storage. `components.rs` and `systems.rs` expose the transitional ECS boundary for future migration.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | Per-contract game state where used by this example. | Keeps small arcade state close to the contract instance. |
| Persistent storage | Player- or world-scoped state where the example needs durable keyed state. | Keeps game progress available across invocations. |
| Temporary storage | Not used. | The examples favor deterministic recalculation over ephemeral caches. |

## Main gameplay flow

1. Call the initialization function to create the starting state.
2. Submit an input action such as movement, jump, flap, rotation, or shoot.
3. Call the tick/update function to run deterministic simulation logic.
4. Query public getters for score, position, active state, or terminal status.
5. Stop when the game-over/completed condition is reached, or reset/reinitialize where supported.

## Cougr APIs used

- `ComponentTrait` and custom component modules document the ECS data boundary.
- `SimpleWorld`, `SimpleQueryBuilder`, `GameApp`, `ScheduleStage`, or `SystemConfig` are used where this transitional example has already adopted the maintained runtime shape.
- Auth, privacy, ZK, and standards APIs are intentionally not used; these arcade examples focus on deterministic game logic.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Transitional code may preserve older storage or scheduling patterns for compatibility reference.
- No authentication, matchmaking, real-time rendering, or production randomness is included.
- One contract instance generally represents one game or one keyed set of player games.
- For new work, prefer the canonical `snake` module split and `GameApp` tick wiring.
