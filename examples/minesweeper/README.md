# Minesweeper

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Minesweeper game built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Purpose and pattern

This example demonstrates a single-player reveal-and-deduce puzzle over a fixed, deterministic
mine layout - no player input or randomness influences where mines are placed, which keeps the
board auditable and reproducible across deployments. It showcases Cougr's `ComponentTrait`
pattern for typed, byte-serializable game state: the board, the (hidden) mine layout, and the
game status are each their own component with manual `serialize`/`deserialize`, demonstrating
explicit byte-layout control for a fixed-size 9×9 grid.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | - | `GameState` | Initializes a 9×9 board with mines placed at fixed, deterministic positions. Overwrites any existing game. |
| `reveal_cell` | `row: u32`, `col: u32` | `RevealResult` | Reveals the cell at `(row, col)`. Returns `success`, `is_mine`, `adjacent_mines`, and a status `message` symbol (`ok`, `invalid`, `revealed`, `boom`, or `over`). |
| `get_state` | - | `GameState` | Current status, revealed-cell count, and remaining safe cells. |
| `get_visible_cell` | `row: u32`, `col: u32` | `VisibleCellState` | Whether the cell is revealed, whether it's a mine (only meaningful once revealed), and its adjacent-mine count. |
| `is_finished` | - | `bool` | Whether the game has reached `Won` or `Lost`. |
| `get_board` | - | `Vec<u32>` | Raw 81-cell board array (row-major); each entry is `0`–`8` for a revealed count, `9` for hidden, or `10` for a revealed mine. |
| `reset_game` | - | `GameState` | Re-initializes the game (equivalent to calling `init_game` again). |

## Architecture overview

There is no `GameApp` tick loop - `reveal_cell` runs all logic synchronously in a fixed
pipeline on each invocation:

```
reveal_cell
  └─ status/bounds/already-revealed checks  → rejects finished games, out-of-bounds
     coordinates, and cells already revealed with the matching `message` symbol
  └─ mine check                              → if the cell holds a mine, marks the cell,
     sets status to Lost, and returns immediately
  └─ adjacency count + board update          → writes the computed adjacent-mine count
     into the board for a safe reveal
  └─ CompletionSystem                        → sets status to Won once every safe cell
     has been revealed
```

`components.rs` defines the data (`BoardComponent`, `MineLayoutComponent`,
`GameStateComponent`, plus the `ECSWorldState` aggregate and the public API types);
`systems.rs` holds the pure functions (`place_mines_deterministic`, `completion_system`);
`lib.rs` owns the single `ECSWorldState` per contract instance and is the only module that
touches contract storage.

## Storage model

The entire game lives under one instance-storage key (`WORLD_KEY`, `symbol_short!("WORLD")`)
as a single `ECSWorldState` struct bundling the board, the mine layout, and the game status.
Instance storage is used - not persistent or temporary - because there is exactly one game per
contract instance and the state must survive for the lifetime of that instance with no
per-entry TTL management.

## Main gameplay flow

1. Deployer (or any caller) calls `init_game()`. The board starts fully hidden and
   `place_mines_deterministic` seeds 10 mines at fixed coordinates across the 9×9 grid.
2. Player calls `reveal_cell(row, col)` for a hidden cell.
3. If the cell is a mine, the board marks it revealed-as-mine, status flips to `Lost`, and the
   call returns `message = boom`. No further reveals are accepted until `reset_game`.
4. If the cell is safe, the contract counts mines among its up-to-8 neighbors, writes that
   count into the board, and increments the revealed-cell counter.
5. After each safe reveal, `CompletionSystem` checks whether all 71 safe cells (81 total minus
   10 mines) have been revealed; if so, status flips to `Won`.
6. Caller reads `get_state`, `get_visible_cell`, or `is_finished` to check progress or
   terminal state; `reset_game` starts over with the same deterministic layout.

## Cougr APIs used

- `cougr_core::component::ComponentTrait` - gives each component (`BoardComponent`,
  `MineLayoutComponent`, `GameStateComponent`) a `component_type()` symbol and explicit byte
  `serialize`/`deserialize`, chosen because the game has one fixed-shape 81-cell grid rather
  than a dynamic entity population that would benefit from `SimpleWorld`/`SimpleQueryBuilder`
  scanning.
- No other `cougr-core` APIs are used. `GameApp`/`ScheduleStage` are unnecessary because
  `reveal_cell` has a single decision point per call with no multi-stage ordering; `SimpleWorld`/
  `SimpleQueryBuilder` are unnecessary because the game has exactly three fixed components and
  never scans entities by type; `auth`, `privacy::stable`/`experimental`, and `ops` standards
  are unused because this example has no multi-party authorization, hidden-information
  disclosure, or pausability/ownership requirements - the entire board state is openly
  readable via `get_board` and `get_visible_cell`.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` - reveal logic is invoked directly
  from contract entrypoints since a single-player puzzle with one decision point per call has
  no need for staged scheduling.
- Mine placement is fixed and deterministic rather than randomized per game; this keeps the
  layout auditable but means every new game (or `reset_game` call) has mines in the same
  positions.
- No flagging system - the contract only supports revealing cells, not marking suspected
  mines.
- No multi-player or wagering support; any caller can call any entrypoint for the single
  shared game instance.
