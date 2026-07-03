# Sudoku

An on-chain Sudoku puzzle built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Overview

This example implements a 9×9 Sudoku puzzle as a Soroban smart contract. The focus is on strict constraint validation — row, column, and 3×3 block rules are enforced on every move — showcasing how `cougr-core` separates validation logic, board mutation, and completion detection into discrete systems.

## ECS Design

### Components

All components derive `Clone + Debug`, are annotated with `#[contracttype]`, and implement `ComponentTrait`.

```rust
pub struct BoardComponent {
    pub cells: Vec<u32>,  // 81 elements, row-major; 0=empty, 1–9=value
}

pub struct FixedCellsComponent {
    pub fixed: Vec<bool>,  // 81 elements; true=immutable (puzzle givens)
}

pub struct GameStatusComponent {
    pub status: u32,  // 0=playing, 1=solved
}

pub struct MoveCountComponent {
    pub moves: u32,  // number of successful placements
}
```

All four components are stored under a single `"WORLD"` key as `ECSWorldState`.

### Systems

Systems run in sequence inside `submit_value`:

1. **InputSystem** — validates that coordinates are in-bounds, the target cell is editable, and the value is 1–9; panics otherwise
2. **PlacementValidationSystem** — checks row, column, and 3×3 block constraints
3. **BoardUpdateSystem** — places the value on the board
4. **EndConditionSystem** — detects puzzle completion (all cells filled, all constraints satisfied)

## Contract API

### Functions

| Function | Description |
|----------|-------------|
| `init_game(puzzle)` | Load a caller-supplied 81-cell puzzle; status = playing |
| `submit_value(row, col, value)` | Place a digit; panics on any invalid input |
| `get_state() → GameState` | Current status and move count |
| `get_cell(row, col) → CellState` | Value and fixed flag for one cell |
| `is_solved() → bool` | True when puzzle is complete |

### Return Types

```rust
pub struct GameState {
    pub status: u32,  // 0=playing, 1=solved
    pub moves: u32,
}

pub struct CellState {
    pub value: u32,   // 0=empty, 1–9
    pub fixed: bool,  // true=immutable
}
```

## Rules

- The board is 9×9, divided into nine 3×3 blocks.
- Fixed cells contain the puzzle's given digits and cannot be overwritten.
- A placement is valid only if the value does not already appear in the same row, column, or 3×3 block.
- The puzzle is solved when all 81 cells are filled and all constraints are satisfied.

## Implementation Patterns

### Constraint Validation

`placement_validation_system` checks three independent constraints — row, column, and block — and returns `false` on the first violation. Each helper scans only the relevant cells, skipping the target cell itself:

```rust
fn check_row(cells: &Vec<u32>, row: u32, skip_col: u32, value: u32) -> bool {
    for c in 0..BOARD_SIZE {
        if c != skip_col && get_cell(cells, row, c) == value {
            return false;
        }
    }
    true
}
```

The block helper derives the top-left corner from integer division:

```rust
let block_row = (row / 3) * 3;
let block_col = (col / 3) * 3;
```

### Completion Detection

`end_condition_system` delegates to `completion_system`, which first scans for any empty cell (fast exit), then verifies every row, column, and block using the same `seen[v]` bitmask pattern — `O(81)` total work regardless of board state.

## Test Coverage

| Category | Tests | What is verified |
|----------|-------|-----------------|
| Initialisation | 5 | Game state, fixed values, empty cells, re-init guard, is_solved at start |
| Move validation | 7 | Valid move, fixed-cell rejection, value 0 and 10 rejected, row/col/block conflict |
| Score tracking | 1 | Move counter increments correctly |
| Completion | 2 | Full solution reaches solved, moves rejected after solved |
| **Total** | **15** | |

## Board Layout

The puzzle is passed to `init_game` as a flat 81-element `Vec<u32>` in row-major order. `0` marks an editable cell; `1–9` marks a fixed given. Example fixture (`.` = editable):

```
     0   1   2   3   4   5   6   7   8
  0  5   3   .   .   7   .   .   .   .
  1  6   .   .   1   9   5   .   .   .
  2  .   9   8   .   .   .   .   6   .
  3  8   .   .   .   6   .   .   .   3
  4  4   .   .   8   .   3   .   .   1
  5  7   .   .   .   2   .   .   .   6
  6  .   6   .   .   .   .   2   8   .
  7  .   .   .   4   1   9   .   .   5
  8  .   .   .   .   8   .   .   7   9
```

Cells are stored row-major: `cells[row * 9 + col]`.

## Development

Requires Rust with `wasm32v1-none` target and [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli).

```bash
# Run tests
cargo test

# Format check
cargo fmt --check

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Build WASM
stellar contract build
```

## Playing the Game

After building the WASM (`stellar contract build`), deploy and play on Testnet:

```bash
# Generate a player identity and fund via Friendbot
stellar keys generate sudoku_player
stellar keys fund sudoku_player --network <NETWORK>

# Deploy the contract
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/sudoku.wasm \
  --network <NETWORK> \
  --source sudoku_player)

# Initialise the puzzle (pass a flat 81-element JSON array; 0=empty, 1–9=fixed)
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source sudoku_player \
  -- init_game \
  --puzzle '[5,3,0,0,7,0,0,0,0,6,0,0,1,9,5,0,0,0,0,9,8,0,0,0,0,6,0,8,0,0,0,6,0,0,0,3,4,0,0,8,0,3,0,0,1,7,0,0,0,2,0,0,0,6,0,6,0,0,0,0,2,8,0,0,0,0,4,1,9,0,0,5,0,0,0,0,8,0,0,7,9]'

# Read the initial state
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source sudoku_player \
  -- get_state

# Read a cell
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source sudoku_player \
  -- get_cell --row 0 --col 2

# Submit a value
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source sudoku_player \
  -- submit_value --row 0 --col 2 --value 4

# Check if solved
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source sudoku_player \
  -- is_solved
```

`submit_value` panics (transaction fails) on any invalid input — the chain rejects it cleanly.
