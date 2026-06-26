# Connect Four

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Connect Four game built with the [Cougr](../../README.md) ECS framework on
Stellar Soroban.

## Purpose and pattern

This example demonstrates a turn-based, perfect-information board game driven by
gravity-based piece placement: a move only specifies a column, and the contract resolves
which row the piece lands in before checking for a win. It showcases Cougr's
`ComponentTrait` pattern for typed, byte-serializable game state — components are plain
Rust structs annotated with `#[contracttype]` that implement `ComponentTrait` directly,
rather than being scanned through `SimpleWorld`/`SimpleQueryBuilder`. This is a
lighter-weight integration suited to a single fixed-shape game state (one board, two
players) with no dynamic entity population.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `player_one: Address`, `player_two: Address` | `GameState` | Initializes an empty 7×6 board; `player_one` moves first. |
| `drop_piece` | `player: Address`, `column: u32` | `DropResult` | Validates the move, drops a piece into the lowest empty row of `column` (0–6), checks for a win/draw, and advances the turn. |
| `get_state` | — | `GameState` | Current board, turn, move count, and status. |
| `get_board` | — | `Vec<u32>` | Raw 42-cell board array (row-major, 0=empty/1=player one/2=player two). |
| `is_valid_column` | `column: u32` | `bool` | Whether `column` is in bounds, not full, and the game is still in progress. |
| `is_finished` | — | `bool` | Whether the game has ended (win or draw). |
| `get_winner` | — | `Option<Address>` | The winning player's address, or `None` if the game is a draw or still in progress. |
| `reset_game` | — | `GameState` | Re-initializes the board with the same two players. |

## Architecture overview

There is no `GameApp` tick loop — `drop_piece` runs all systems synchronously in a fixed
pipeline on each invocation:

```
drop_piece
  └─ validation_system     → rejects out-of-turn, out-of-bounds, full-column, or post-game moves
  └─ gravity_system        → finds the lowest empty row in the target column
  └─ execution_system      → places the piece and increments the move counter
  └─ win_detection_system  → scans the board for horizontal/vertical/diagonal 4-in-a-row
  └─ draw_system           → marks the game a draw if the board fills with no winner
  └─ turn_system           → advances to the other player if the game is still active
```

Each system is a pure function over `components.rs` types (`BoardComponent`,
`PlayerComponent`, `GameStateComponent`, bundled into `ECSWorldState`); `lib.rs` owns that
aggregate and is the only module that touches contract storage.

## Storage model

The entire game lives under one instance-storage key (`WORLD_KEY`,
`symbol_short!("WORLD")`) as a single `ECSWorldState` struct bundling the board, both
player addresses, and the game-state component (turn, move count, status, last move).
Instance storage is used — not persistent or temporary — because there is exactly one game
per contract instance and the state must survive for the lifetime of that instance with no
per-entry TTL management.

## Main gameplay flow

1. Deployer calls `init_game(player_one, player_two)`; the board is seeded empty,
   `player_one` to move, status in-progress.
2. `player_one` calls `drop_piece(player, column)`. The contract checks turn order, column
   bounds, and that the column isn't full.
3. On a legal move: `gravity_system` finds the lowest empty row in that column,
   `execution_system` places the piece there and increments the move count,
   `win_detection_system` scans the whole board for a 4-in-a-row, and `draw_system` marks
   the game a draw if all 42 cells are filled with no winner.
4. If the game is still in progress, `turn_system` switches to the other player; the
   updated `ECSWorldState` is written back to instance storage.
5. Players alternate `drop_piece` calls until `is_finished` returns `true`. Either player
   reads `get_winner` to see who won (or `None` for a draw).
6. `reset_game` can be called at any point to start a fresh game with the same two players.

## Cougr APIs used

- `cougr_core::component::ComponentTrait` — gives each component (`BoardComponent`,
  `GameStateComponent`) a `component_type()` symbol and explicit byte-level
  `serialize`/`deserialize`. This was chosen because the game has one fixed-shape
  `ECSWorldState` per contract instance (a single board, two fixed player slots, one game-
  state record) rather than a dynamic population of entities that would benefit from
  `SimpleWorld`/`SimpleQueryBuilder` scanning — per the guidance in
  [`EXAMPLE_STANDARD.md` §8](../EXAMPLE_STANDARD.md#8-cougr-api-usage-guidance), `SimpleWorld`
  and `SimpleQueryBuilder` are for examples that store and query entities by component type,
  which doesn't apply here. `PlayerComponent` is also `#[contracttype]` but does not
  implement `ComponentTrait`, since it is embedded directly in `ECSWorldState` rather than
  serialized standalone.
- No `GameApp`, `ScheduleStage`, `auth`, `privacy::stable`/`experimental`, or `ops`
  standards are used — `drop_piece` has exactly one decision point per call, so there is no
  multi-stage tick to schedule, no hidden information to commit-reveal, and no
  pausability/ownership requirement for a two-player, always-on game.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` — game logic is invoked
  directly from contract entrypoints rather than through a tick-based scheduler, since a
  turn-based game with one decision point per call does not need staged scheduling.
- No timeout/forfeit mechanism for an unresponsive player.
- `win_detection_system` re-scans the entire board on every move rather than checking only
  the cell that was just placed; this is simple and correct but not the most efficient
  approach for a larger board.
- No spectator or replay API beyond the read-only getters.
