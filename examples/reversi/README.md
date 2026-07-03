# Reversi

An on-chain Reversi (Othello) game built with the [Cougr](../../README.md) ECS framework on Stellar Soroban.

## Overview

This example implements a complete Reversi (Othello) game as a Soroban smart contract. The focus is on the smart contract logic, showcasing how `cougr-core` simplifies on-chain turn-based game development with isolated systems and typed components.

## ECS Design

### Components

All components derive `Clone + Debug`, annotated with `#[contracttype]` for XDR-serialisation, and implement `ComponentTrait` for Cougr's byte-level storage layer.

```rust
pub struct BoardComponent {
    pub cells: Vec<u32>,  // 64 elements, row-major; 0=empty, 1=black, 2=white
    pub width: u32,       // always 8
    pub height: u32,      // always 8
}

pub struct TurnComponent {
    pub current_player: u32, // 1=black, 2=white
    pub pass_count: u32,     // 0=normal, 1=opponent skipped, 2=both locked → game ends
}

pub struct GameStatusComponent {
    pub status: u32,  // 0=active, 1=finished
}

pub struct ScoreComponent {
    pub black_count: u32,
    pub white_count: u32,
}
```

The entire game state is stored under a single `"WORLD"` key as `ECSWorldState`:

```rust
pub struct ECSWorldState {
    pub board:      BoardComponent,
    pub turn:       TurnComponent,
    pub status:     GameStatusComponent,
    pub score:      ScoreComponent,
    pub player_one: Address,  // plays BLACK (1)
    pub player_two: Address,  // plays WHITE (2)
}
```

### Systems

Systems run in sequence inside `submit_move`:

1. **MoveValidationSystem** — rejects occupied cells and moves with no bracketed pieces
2. **FlipResolutionSystem** — places piece and flips all bracketed opponent pieces in 8 directions
3. **ScoringSystem** — recomputes piece counts from board state
4. **TurnSystem + PassSystem** — advances to opponent; auto-skips if opponent has no legal move; signals game end when both have no moves
5. **EndConditionSystem** — sets status=1 when pass_count≥2 or board is full

## Contract API

### Functions

| Function | Description |
|----------|-------------|
| `init_game(player_one, player_two)` | Initialise board; Black moves first |
| `submit_move(player, row, col)` | Place piece; panics on illegal move or wrong turn |
| `get_state() → GameState` | Current player, pass_count state, active/finished |
| `get_board() → BoardState` | Raw cell array (row-major, 0-indexed) |
| `get_score() → ScoreState` | Piece counts and winner (0=ongoing, 1=black, 2=white, 3=draw) |

### Return Types

```rust
pub struct GameState {
    pub current_player: u32,  // 1=black, 2=white
    pub pass_count: u32,      // 0, 1, or 2
    pub status: u32,          // 0=active, 1=finished
}

pub struct BoardState {
    pub cells: Vec<u32>,  // 64 values, row-major
    pub width: u32,       // 8
    pub height: u32,      // 8
}

pub struct ScoreState {
    pub black_count: u32,
    pub white_count: u32,
    pub winner: u32,  // 0=ongoing, 1=black, 2=white, 3=draw
}
```

## Rules

- Black moves first. Players alternate turns.
- A move must flip at least one opponent piece in a straight line (horizontal, vertical, or diagonal).
- If a player has no legal move, their turn is automatically skipped (`pass_count` = 1).
- Game ends when both players have no legal moves (`pass_count` = 2) or the board is full.
- The player with more pieces wins.

## Implementation Patterns

### Flip Algorithm

Reversi's core mechanic: a move is legal only if it brackets at least one opponent piece in a straight line, ending with one of the mover's own pieces. The contract checks all 8 directions using a direction-vector table:

```rust
const DIRS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),   // NW  N  NE
    ( 0, -1),          ( 0, 1),   // W      E
    ( 1, -1), ( 1, 0), ( 1, 1),   // SW  S  SE
];
```

`flips_in_dir` walks one step at a time from `(row+dr, col+dc)`:
- Counts consecutive opponent pieces.
- Returns the count if it finds one of the player's own pieces at the end (bracket found).
- Returns 0 if it hits an empty cell or the board edge (no bracket).

`FlipResolutionSystem` then re-walks each direction where `flips_in_dir > 0` and overwrites those cells with the mover's colour. `Vec::set` on a `soroban_sdk::Vec` mutates in place, so the board component is taken by value and returned.

### Pass State Machine

`pass_count` is recomputed from scratch every turn — it is not accumulated:

```
After every move:
  if opponent has legal moves     → pass_count = 0  (normal alternation)
  elif current has legal moves    → pass_count = 1  (opponent auto-passed, current continues)
  else                            → pass_count = 2  (both locked → EndConditionSystem ends game)
```

This is handled by `TurnSystem` delegating to `PassSystem` when the opponent has no moves.

## Test Coverage

| Category | Tests | What is verified |
|----------|-------|-----------------|
| Initialisation | 3 | Board layout, opening score (2-2), initial turn state |
| Move validation | 2 | Occupied-cell rejection, no-flip rejection |
| Flip mechanics | 3 | Horizontal, vertical, and diagonal flips |
| Score tracking | 1 | Counts update correctly after a flip |
| Turn management | 2 | Turn alternation, wrong-player rejection |
| Pass / sequence | 2 | Normal pass_count=0, multi-move game stays active |
| Re-initialisation | 1 | Second `init_game` call is rejected |
| **Total** | **14** | |

## Board Layout

Cells are stored row-major: `cells[row * 8 + col]`.

```
     0   1   2   3   4   5   6   7
  0  .   .   .   .   .   .   .   .
  1  .   .   .   .   .   .   .   .
  2  .   .   .   .   .   .   .   .
  3  .   .   .   W   B   .   .   .
  4  .   .   .   B   W   .   .   .
  5  .   .   .   .   .   .   .   .
  6  .   .   .   .   .   .   .   .
  7  .   .   .   .   .   .   .   .
```

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
# Generate two player identities and fund them via Friendbot
stellar keys generate reversi_black
stellar keys generate reversi_white
stellar keys fund reversi_black --network <NETWORK>
stellar keys fund reversi_white --network <NETWORK>

# Deploy the contract
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/reversi.wasm \
  --network <NETWORK> \
  --source reversi_black)

# Initialise (reversi_black plays Black, reversi_white plays White)
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_black \
  -- init_game \
  --player_one reversi_black \
  --player_two reversi_white

# Black places at row=3, col=2 (flips (3,3) horizontally)
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_black \
  -- submit_move \
  --player reversi_black \
  --row 3 --col 2

# Check the board after the move
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_black \
  -- get_board
# → {"cells":[0,0,...,1,1,1,0,...,1,2,0,...],"width":8,"height":8}
# (3,2)=1 (3,3)=1 (3,4)=1 — three Black pieces in a row

# Check whose turn it is
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_black \
  -- get_state
# → {"current_player":2,"pass_count":0,"status":0}
# current_player=2 → White's turn, game active

# Check the score
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_black \
  -- get_score
# → {"black_count":4,"white_count":1,"winner":0}
# winner=0 → game ongoing

# White responds at row=2, col=3
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> --source reversi_white \
  -- submit_move \
  --player reversi_white \
  --row 2 --col 3
```

`submit_move` panics (transaction fails) on an illegal move or wrong turn — the chain rejects it cleanly. `get_score` returns `winner = 0` while the game is active and `1` (Black), `2` (White), or `3` (draw) once it ends.
