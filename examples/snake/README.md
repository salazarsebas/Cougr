# Snake On-Chain Game

**Classification: Canonical example.** This is the maintained arcade reference for Cougr examples. New arcade contracts should copy its `GameApp` wiring, `components.rs` / `systems.rs` split, README shape, and test coverage.

## Purpose and pattern

Snake demonstrates a deterministic arcade loop on Soroban using `cougr-core`'s basic ECS and `GameApp` tick model. The contract keeps persistent game state on chain, stores ECS entities in a `SimpleWorld`, and runs ordered systems for movement, collision, growth, and food spawning.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | none | `()` | Initializes a new game on the default 10×10 grid. |
| `init_game_with_size` | `grid_size: i32` | `()` | Initializes a new game on a custom square grid. |
| `change_direction` | `direction: u32` | `bool` | Changes the snake direction (`0` up, `1` down, `2` left, `3` right); returns `false` for invalid values, reversals, or game-over state. |
| `update_tick` | none | `()` | Advances the game by one `GameApp` tick. |
| `get_score` | none | `u32` | Returns the current score. |
| `check_game_over` | none | `bool` | Returns whether the game has reached a terminal state. |
| `get_head_pos` | none | `(i32, i32)` | Returns the current snake-head position. |
| `get_snake_length` | none | `u32` | Returns the number of snake entities. |
| `get_food_pos` | none | `(i32, i32)` | Returns the current food position. |
| `get_snake_positions` | none | `Vec<(i32, i32)>` | Returns all snake segment positions. |
| `get_grid_size` | none | `i32` | Returns the configured grid size. |

## Architecture overview

```text
contract entrypoint
  ├─ loads GameState + SimpleWorld from persistent storage
  ├─ builds a GameApp around the world
  ├─ schedules systems by stage
  │   ├─ Update: move_snake
  │   └─ PostUpdate: self_collision -> food_collision
  └─ writes GameState + SimpleWorld back to storage
```

- `lib.rs` contains the Soroban contract entrypoints, storage access, and `GameApp` wiring.
- `components.rs` contains serializable ECS components such as `Position`, `DirectionComponent`, `SnakeHead`, `SnakeSegment`, and `Food`.
- `systems.rs` contains reusable game systems for movement, direction validation, collision checks, growth, and food spawning.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | none | The example does not need contract-wide configuration shared across games. |
| Persistent storage | `state: GameState`, `world: SimpleWorld` | Game progress must survive across transactions. `GameState` stores compact scalar data; `SimpleWorld` stores entities and component bytes. |
| Temporary storage | none | No per-ledger cache is needed for deterministic gameplay. |

Within the `SimpleWorld`, dense components such as positions and directions use table-style access, while marker-style components such as food/head/segment are queried as needed.

## Main gameplay flow

1. A player calls `init_game` or `init_game_with_size`.
2. Startup systems spawn the snake head at the grid center and create one food entity.
3. The player calls `change_direction` to submit a valid non-reversing input.
4. The player or a relayer calls `update_tick`.
5. `GameApp` runs movement first, then collision and food checks.
6. A wall/self collision sets `game_over`; eating food grows the snake, increments score, and spawns new food.
7. Query functions expose score, positions, grid size, and terminal state.

## Cougr APIs used

| API | Why it is used |
|---|---|
| `GameApp` | Provides the maintained arcade-loop pattern and owns scheduled system execution per tick. |
| `ScheduleStage` / `SystemConfig` | Ensures movement runs before post-update collision and food systems. |
| `SimpleWorld` | Stores snake, food, and component data in a Soroban-serializable ECS container. |
| `SimpleQueryBuilder` | Scans entities by component type for food, head, and segment queries. |
| `ComponentTrait` | Gives each custom component deterministic serialization and a stable component type. |

This example does not use Cougr auth, privacy, ZK, or standards modules because Snake is intentionally a single-player arcade-loop reference.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Food spawning is deterministic and suitable for examples, not adversarial randomness.
- There is one game state per contract instance.
- No authentication or ownership model is included.
- Rendering and real-time scheduling are out of scope; callers drive ticks through contract invocations.
