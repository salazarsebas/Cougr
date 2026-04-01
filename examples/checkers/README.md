# Checkers — Cougr ECS Example

A fully on-chain two-player Checkers game implemented as a Soroban smart
contract. This example demonstrates how the Cougr Entity Component System
(ECS) model applies to a board game with non-trivial rule enforcement —
combining grid movement, mandatory captures, king promotion, and win
detection inside a single deterministic contract.

---

## Why Checkers?

Tic-Tac-Toe establishes the basics of turn management and a fixed board.
Checkers is the natural next step:

| Feature | Tic-Tac-Toe | Checkers |
|---|---|---|
| Fixed grid | ✓ | ✓ |
| Two-player turns | ✓ | ✓ |
| Piece movement rules | — | ✓ |
| Capture mechanics | — | ✓ |
| Forced-move enforcement | — | ✓ |
| Piece promotion | — | ✓ |
| Multi-hop chain captures | — | ✓ |
| Stalemate detection | — | ✓ |

---

## Board Layout

Standard 8×8 English Draughts. Only dark squares (where `row + col` is odd)
are ever occupied.

```
     col 0   1   2   3   4   5   6   7
row 0  [ ] [P1] [ ] [P1] [ ] [P1] [ ] [P1]
row 1  [P1] [ ] [P1] [ ] [P1] [ ] [P1] [ ]
row 2  [ ] [P1] [ ] [P1] [ ] [P1] [ ] [P1]
row 3  [ ] [ ] [ ] [ ] [ ] [ ] [ ] [ ]
row 4  [ ] [ ] [ ] [ ] [ ] [ ] [ ] [ ]
row 5  [P2] [ ] [P2] [ ] [P2] [ ] [P2] [ ]
row 6  [ ] [P2] [ ] [P2] [ ] [P2] [ ] [P2]
row 7  [P2] [ ] [P2] [ ] [P2] [ ] [P2] [ ]
```

**Piece encoding** (stored as `i32` in a flat 64-element vector):

| Value | Meaning |
|---|---|
| `0` | Empty |
| `1` | Player One man |
| `2` | Player One king |
| `-1` | Player Two man |
| `-2` | Player Two king |

Player One moves from row 0 toward row 7 (+1 row per step).  
Player Two moves from row 7 toward row 0 (−1 row per step).  
Kings may move diagonally in all four directions.

---

## ECS Architecture

### Components

| Component | Fields | Purpose |
|---|---|---|
| `BoardComponent` | `cells: Vec<i32>` | 8×8 flat grid of piece values |
| `TurnComponent` | `current_player: u32`, `move_number: u32` | Whose turn it is and how many moves have been played |
| `GameStatusComponent` | `status: GameStatus`, `winner: u32` | Active / Finished and optional winner (1 or 2) |
| `ChainCapture` *(internal)* | `row: u32`, `col: u32` | Tracks the landing square during a multi-hop capture sequence |

### Systems

| System | Responsibility |
|---|---|
| `MoveValidationSystem` | Checks diagonal legality, occupancy, and bounds for both steps and jumps |
| `CaptureSystem` | Identifies the jumped piece, removes it, and detects further chain-capture options |
| `PromotionSystem` | Promotes a man to king when it reaches the opponent's back rank |
| `TurnSystem` | Advances the turn after a non-capturing move or when no further captures exist from the landing square; holds the turn during chain captures |
| `EndConditionSystem` | Declares a winner when one side has no pieces or no legal moves |

---

## Contract API

```rust
/// Initialise a new game.
fn init_game(env: Env, player_one: Address, player_two: Address)

/// Submit a move from (from_row, from_col) to (to_row, to_col).
fn submit_move(env: Env, player: Address, from_row: u32, from_col: u32, to_row: u32, to_col: u32)

/// Return the full game state snapshot.
fn get_state(env: Env) -> GameState

/// Return the current board cells (64 values, row-major order).
fn get_board(env: Env) -> BoardState

/// Return the Address of the player whose turn it currently is.
fn get_current_player(env: Env) -> Address
```

### Error codes

| Error | Code | Meaning |
|---|---|---|
| `AlreadyInitialised` | 1 | `init_game` called more than once |
| `NotInitialised` | 2 | Any call before `init_game` |
| `NotAPlayer` | 3 | Caller is not `player_one` or `player_two` |
| `WrongTurn` | 4 | Caller is the correct player but it is not their turn |
| `NotYourPiece` | 5 | Source square is empty or owned by the opponent |
| `DestinationOccupied` | 6 | Target square is already occupied |
| `IllegalMove` | 7 | Move is not a legal diagonal step or jump |
| `MustCapture` | 8 | A capture is available but a non-capture move was attempted |
| `GameOver` | 9 | The game has already ended |
| `OutOfBounds` | 10 | Row or column index ≥ 8 |
| `ChainCapturePieceMismatch` | 11 | During a chain capture the origin square was not the chain square |
| `NotDarkSquare` | 12 | Destination square is a light square (row + col is even) |

---

## Rules Implemented

1. **Diagonal movement only** — men move one square diagonally forward; kings
   move one square diagonally in any direction.
2. **Captures (jumps)** — a piece jumps over an adjacent opponent piece into
   the empty square beyond. The captured piece is removed immediately.
3. **Forced captures** — if any capture is available for the current player,
   they *must* make a capture. A non-capture step is rejected with
   `MustCapture`.
4. **Chain captures (multi-hop)** — after a capture, if the landing piece can
   continue capturing, the same player must do so. The turn is held and the
   active square is tracked via `ChainCapture`. The player may only move the
   same piece until no further captures are available.
5. **Promotion** — a man reaching the opponent's back rank (row 7 for Player
   One, row 0 for Player Two) is immediately promoted to a king. Promotion
   happens before chain-capture detection, so a newly crowned king may
   continue capturing if further opportunities exist.
6. **Win condition** — a player wins when the opponent has no pieces remaining
   on the board, or when the opponent has no legal move (step or capture)
   available on their turn.

---

## Getting Started

### Prerequisites

- Rust 1.70 or newer
- `wasm32v1-none` target: `rustup target add wasm32v1-none`
- Stellar CLI: `cargo install --locked stellar-cli --features opt`

### Run the tests

```bash
cd examples/checkers
cargo test
```

### Check formatting and lints

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Build the Soroban WASM contract

```bash
stellar contract build
```

The compiled WASM artefact is written to:

```
target/wasm32v1-none/release/checkers.wasm
```

### Deploy to Testnet (optional)

```bash
# Generate or reuse an identity
stellar keys generate --global alice --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/checkers.wasm \
  --source alice \
  --network testnet \
  --alias checkers_contract
```

### Invoke on Testnet

```bash
# Initialise a game
stellar contract invoke \
  --id checkers_contract \
  --source alice \
  --network testnet \
  -- init_game \
  --player_one <PLAYER_ONE_ADDRESS> \
  --player_two <PLAYER_TWO_ADDRESS>

# Submit a move
stellar contract invoke \
  --id checkers_contract \
  --source alice \
  --network testnet \
  -- submit_move \
  --player <PLAYER_ADDRESS> \
  --from_row 2 --from_col 1 \
  --to_row 3 --to_col 0

# Read the current board
stellar contract invoke \
  --id checkers_contract \
  --network testnet \
  -- get_board
```

---

## Project Layout

```
examples/checkers/
├── Cargo.toml          # Package manifest and dependency pinning
├── README.md           # This file
└── src/
    ├── lib.rs          # Contract implementation (components + systems + API)
    └── test.rs         # Integration test suite
```

---

## Test Coverage

| Scenario | Test |
|---|---|
| Standard starting position | `test_init_sets_standard_start_position` |
| Double-init rejection | `test_double_init_fails` |
| State before init | `test_get_state_before_init_fails` |
| Legal diagonal step | `test_legal_diagonal_step_advances_piece` |
| Turn advancement | `test_turn_advances_after_step` |
| Light-square rejection | `test_move_to_light_square_rejected` |
| Wrong-piece rejection | `test_move_wrong_piece_rejected` |
| Empty-square move rejection | `test_move_empty_square_rejected` |
| Horizontal move rejection | `test_horizontal_move_rejected` |
| Backward move (man) rejection | `test_backward_move_man_rejected` |
| Out-of-bounds rejection | `test_out_of_bounds_rejected` |
| Wrong-turn rejection | `test_wrong_turn_player_rejected` |
| Unknown address rejection | `test_unknown_address_rejected` |
| Capture execution + removal | `test_capture_removes_opponent_piece` |
| Forced-capture enforcement | `test_forced_capture_prevents_step` |
| King promotion at back rank | `test_man_promoted_to_king_at_back_rank` |
| Win detection (no pieces) | `test_win_when_opponent_has_no_pieces` |
| Move after game over | `test_move_after_game_over_rejected` |
| King backward movement | `test_king_can_move_backward` |
| Board size invariant | `test_get_board_returns_64_cells` |
| Current-player query | `test_get_current_player_switches_each_turn` |

---

## Design Notes

### Storage Keys

Soroban persistent storage is keyed by `Symbol`. This contract uses five
top-level keys:

| Key | Contents |
|---|---|
| `BOARD` | `BoardComponent` (64 cells) |
| `TURN` | `TurnComponent` |
| `STATUS` | `GameStatusComponent` |
| `P1` | `Address` of Player One |
| `P2` | `Address` of Player Two |
| `CHAIN` | `ChainCapture` (present only during a multi-hop sequence) |

The `CHAIN` key is absent when no multi-hop is in progress. Its presence is
the signal to `TurnSystem` that the current turn is not yet complete.

### `no_std` Compatibility

The contract is compiled with `#![no_std]` as required by Soroban. All
collections use `soroban_sdk::Vec` rather than `std::vec::Vec`. Internal
helper logic that needs small fixed-size arrays uses a local `SmallVec4<T>`
type backed by a stack-allocated `[T; 4]` array.

### Capture Validation Strategy

Capture legality is checked by `MoveValidationSystem` against the list
returned by `legal_captures`. This list is also used to:

- determine whether a forced capture exists anywhere on the board
  (`any_capture_available`),
- detect chain-capture continuation opportunities after a jump lands.

This avoids duplicating directional logic and keeps the validation surface
small and testable.

---

## Relation to Other Examples

| Example | What it adds over this one |
|---|---|
| `tic_tac_toe` | Simpler board, no movement or capture |
| `checkers` (this) | Grid movement, captures, forced rules, promotion, chains |
| `chess` | More piece types, complex movement geometry, check/checkmate |
| `battleship` | Hidden state, commit-reveal, coordinate bombing |

Checkers occupies the middle of the complexity ladder: rich enough to
demonstrate real rule enforcement without the combinatorial complexity of
chess.

---

## License

MIT OR Apache-2.0