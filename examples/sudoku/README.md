# Sudoku

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

An on-chain Sudoku puzzle built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Purpose and pattern

This example demonstrates a single-player constraint-validation puzzle where moves are
checked against row/column/3×3-block rules rather than against an opponent. It showcases
Cougr's `ComponentTrait` pattern for typed, byte-serializable game state: each piece of
state (board, fixed-cell mask, status, move counter) is its own component with manual
`serialize`/`deserialize`, demonstrating explicit byte-layout control for a fixed-size grid.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_game` | `puzzle: Vec<u32>` | — | Seeds an empty 9×9 board from an 81-cell array (0=empty, 1–9=given). Cells with a non-zero starting value become fixed. Panics if already initialized, the length isn't 81, or a value exceeds 9. |
| `submit_value` | `row: u32`, `col: u32`, `value: u32` | — | Validates and places `value` at `(row, col)`. Panics if the puzzle is solved, the cell is fixed/out-of-bounds, the value is outside 1–9, or the placement violates a row/column/block constraint. |
| `get_state` | — | `GameState` | Current status (0=playing, 1=solved) and move count. |
| `get_cell` | `row: u32`, `col: u32` | `CellState` | Current value at the cell and whether it is a fixed given. |
| `is_solved` | — | `bool` | Whether the puzzle is fully and correctly filled. |

## Architecture overview

`submit_value` runs a fixed validation/update pipeline synchronously on each call — there is
no `GameApp` tick loop:

```
submit_value
  └─ InputValidationSystem        → bounds check, rejects fixed cells, rejects out-of-range values
  └─ PlacementValidationSystem    → checks row, column, and 3×3 block constraints
  └─ BoardUpdateSystem            → writes the value into the board component
  └─ CompletionSystem             → re-validates every row/column/block for full-board completion
  └─ EndConditionSystem           → sets status=solved when CompletionSystem passes
```

`components.rs` defines the data (`BoardComponent`, `FixedCellsComponent`,
`GameStatusComponent`, `MoveCountComponent`); `systems.rs` holds the pure validation and
update functions; `lib.rs` owns the single `ECSWorldState` aggregate and is the only module
that touches contract storage.

## Storage model

The full puzzle state lives under one instance-storage key (`WORLD_KEY`,
`symbol_short!("WORLD")`) as a single `ECSWorldState` bundling the board, the fixed-cell
mask, status, and move count. Instance storage is used because there is exactly one puzzle
per contract instance with no per-entry expiry needed; persistent or temporary storage would
add TTL bookkeeping this single-instance puzzle doesn't need.

## Main gameplay flow

1. Deployer calls `init_game(puzzle)` with an 81-cell array representing the starting
   grid; non-zero cells are marked fixed and cannot be overwritten.
2. Player calls `submit_value(row, col, value)` for an empty cell. The contract checks the
   cell isn't fixed, the value is 1–9, and the placement doesn't repeat a value in the same
   row, column, or 3×3 block.
3. On a legal placement, the board updates and the move counter increments.
4. After every placement, `CompletionSystem` re-checks all 9 rows, 9 columns, and 9 blocks.
   When the board is completely and correctly filled, status flips to solved.
5. Caller reads `is_solved` or `get_state` to check completion; `get_cell` reads individual
   cells (e.g., to render the board off-chain).

## Cougr APIs used

- `cougr_core::component::ComponentTrait` — gives each component
  (`BoardComponent`, `FixedCellsComponent`, `GameStatusComponent`, `MoveCountComponent`) a
  `component_type()` symbol and explicit byte `serialize`/`deserialize`, chosen because the
  puzzle has one fixed-shape 81-cell grid rather than a dynamic entity population that would
  benefit from `SimpleWorld`/`SimpleQueryBuilder` scanning.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Does not use `GameApp`, `ScheduleStage`, or `SimpleWorld` — validation runs directly from
  contract entrypoints since a single-player puzzle with one decision point per call has no
  need for staged scheduling.
- `init_game` accepts any caller and does not verify the supplied puzzle has a unique
  solution; puzzle generation/validation is left to the caller.
- No undo, hint, or partial-progress save beyond the committed board state.
