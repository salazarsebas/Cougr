# Connect Four On-Chain Game

A fully functional Connect Four game implemented as a Soroban smart contract on the Stellar blockchain, demonstrating the **Cougr-Core** ECS (Entity Component System) framework for on-chain gaming.

|                 |                                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------- |

## Why Cougr-Core?

Cougr-Core provides an ECS architecture that simplifies on-chain game development. Here's how it compares to vanilla Soroban:

| Aspect                 | Vanilla Soroban                       | With Cougr-Core                                           |
| ---------------------- | ------------------------------------- | --------------------------------------------------------- |
| **Data Serialization** | Manual byte packing/unpacking         | `ComponentTrait` with type-safe `serialize`/`deserialize` |
| **Code Organization**  | Monolithic contract logic             | Modular components and systems                            |
| **Type Safety**        | Runtime errors from format mismatches | Compile-time checking via traits                          |
| **Reusability**        | Copy-paste between projects           | Shared component interfaces across games                  |
| **Extensibility**      | Refactor existing code                | Add new systems without modification                      |

### ComponentTrait Integration

All game components implement `cougr_core::component::ComponentTrait`:

```rust
impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes { /* ... */ }
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> { /* ... */ }
}
```

### ECS System Pattern

Game logic is organized into discrete systems:

| System               | Responsibility                                          |
| -------------------- | ------------------------------------------------------- |
| `validation_system`  | Enforces game rules (turn order, column bounds, full)   |
| `gravity_system`    | Finds lowest empty row for piece placement              |
| `execution_system`   | Places piece on the board                               |
| `win_detection_system` | Checks horizontal, vertical, and diagonal wins        |
| `draw_system`        | Detects full board with no winner                       |
| `turn_system`        | Manages turn transitions                                |

## Features

| Feature                | Description                                                    |
| ---------------------- | -------------------------------------------------------------- |
| Two-player gameplay    | Uses Stellar addresses for player identification               |
| Turn-based mechanics   | Player One goes first, enforced turn order                     |
| Gravity-based placement| Pieces automatically fall to lowest available row              |
| Win detection          | Horizontal, vertical, and both diagonal patterns               |
| Draw detection         | Recognizes full board with no winner                           |
| Move validation        | Rejects invalid columns, full columns, wrong turns             |
| Game reset             | Restart with same players                                      |
| Last move tracking     | Tracks which column was last played                            |

## Prerequisites

| Requirement | Version               |
| ----------- | --------------------- |
| Rust        | 1.70.0+               |
| Stellar CLI | 25.0.0+ (recommended) |

```bash
cargo install stellar-cli
```

## Building

```bash
# Build for testing
cargo build

# Build optimized WASM
stellar contract build
```

## Testing

```bash
cargo test
```

| Test Category         | Count | Coverage                                              |
| --------------------- | ----- | ----------------------------------------------------- |
| Initialization        | 2     | Game setup, board retrieval                           |
| Legal token drop      | 3     | First move, gravity stacking, alternating turns       |
| Full column rejection | 2     | Column filled completely, validation after fill       |
| Wrong turn rejection  | 2     | Out-of-turn play, invalid player                      |
| Horizontal win        | 3     | Bottom row, middle row, any row                       |
| Vertical win          | 2     | Player 1 vertical, Player 2 vertical                  |
| Diagonal win          | 2     | Positive slope, negative slope                        |
| Draw detection        | 2     | Full board scenarios                                  |
| Edge cases            | 7     | Out of bounds, game over, reset, winner tracking, etc |
| **Total**             | **25**| **All passing**                                        |

## Contract API

### Functions

| Function       | Parameters                             | Returns       | Description                          |
| -------------- | -------------------------------------- | ------------- | ------------------------------------ |
| `init_game`    | `player_one: Address, player_two: Address` | `GameState` | Initialize new game                  |
| `drop_piece`   | `player: Address, column: u32`         | `DropResult`  | Drop piece in column (0-6)           |
| `get_state`    | -                                      | `GameState`   | Get current game state               |
| `get_board`    | -                                      | `Vec<u32>`    | Get flattened board array            |
| `is_valid_column` | `column: u32`                       | `bool`        | Check if column is valid and not full|
| `is_finished`  | -                                      | `bool`        | Check if game is over                |
| `get_winner`   | -                                      | `Option<Address>` | Get winner's address if game over |
| `reset_game`   | -                                      | `GameState`   | Reset with same players              |

### Board Layout

```
Columns: 0  1  2  3  4  5  6
        ┌───────────────────┐
Row 0   │ .  .  .  .  .  .  . │
Row 1   │ .  .  .  .  .  .  . │
Row 2   │ .  .  .  .  .  .  . │
Row 3   │ .  .  .  .  .  .  . │
Row 4   │ .  .  .  .  .  .  . │
Row 5   │ .  .  .  .  .  .  . │
        └───────────────────┘
```

- **Rows**: 6 (indexed 0-5, top to bottom)
- **Columns**: 7 (indexed 0-6, left to right)
- **Cell values**: 0 = Empty, 1 = Player One, 2 = Player Two

### Data Structures

**GameState**
| Field                | Type         | Description                                          |
| -------------------- | ------------ | ---------------------------------------------------- |
| `board`              | `Vec<u32>`   | Flattened 7×6 board (row-major order)                |
| `rows`               | `u32`        | Number of rows (6)                                   |
| `cols`               | `u32`        | Number of columns (7)                                |
| `player_one`         | `Address`    | Player One's address                                 |
| `player_two`         | `Address`    | Player Two's address                                 |
| `is_player_one_turn` | `bool`       | True if Player One's turn                            |
| `move_count`         | `u32`        | Total moves made                                     |
| `status`             | `u32`        | 0=InProgress, 1=P1Wins, 2=P2Wins, 3=Draw             |
| `last_move_col`      | `Option<u32>`| Column index of last move                            |

**DropResult**
| Field         | Type        | Description                           |
| ------------- | ----------- | ------------------------------------- |
| `success`     | `bool`      | Whether move succeeded                |
| `game_state`  | `GameState` | Updated game state                    |
| `message`     | `Symbol`    | Status code                           |
| `row_placed`  | `Option<u32>`| Row where piece landed (if success)  |

### Error Messages

| Code       | Meaning                                      |
| ---------- | -------------------------------------------- |
| `ok`       | Move successful                              |
| `invalid`  | Column out of bounds (not 0-6)               |
| `full`     | Column is already full                       |
| `notturn`  | Not the player's turn                        |
| `notplay`  | Address is not a registered player           |
| `gameover` | Game has already ended                       |

## Architecture

```text
ECSWorldState
├── BoardComponent         (entity_id: 0)
│   └── cells: Vec<u32> [42 cells - 7 columns × 6 rows]
├── PlayerComponent        (entity_id: 1)
│   ├── player_one: Address
│   └── player_two: Address
├── GameStateComponent     (entity_id: 2)
│   ├── is_player_one_turn: bool
│   ├── move_count: u32
│   ├── status: u32
│   ├── last_move_col: Option<u32>
│   └── entity_id: u32
└── next_entity_id: u32
```

### Win Detection Algorithm

The win detection system checks all four directions from every occupied cell:

1. **Horizontal**: Check 4 consecutive cells in the same row
2. **Vertical**: Check 4 consecutive cells in the same column
3. **Diagonal (positive slope)**: Check diagonal from bottom-left to top-right
4. **Diagonal (negative slope)**: Check diagonal from top-left to bottom-right

```rust
// Example: Horizontal check
fn check_horizontal(board: &BoardComponent, env: &Env, row: u32, col: u32, cell: u32) -> bool {
    if col + 3 >= COLS { return false; }
    for i in 0..4 {
        if board.get_cell(env, row, col + i) != cell {
            return false;
        }
    }
    true
}
```

## Gameplay Example

```text
Initial State (empty board):
┌───────────────────┐
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
└───────────────────┘

After moves: P1→col3, P2→col4, P1→col3, P2→col4, P1→col3, P2→col4
┌───────────────────┐
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  2  1  .  . │
│ .  .  .  1  2  .  . │
└───────────────────┘

Player 1 wins with vertical line in column 3:
┌───────────────────┐
│ .  .  .  .  .  .  . │
│ .  .  .  .  .  .  . │
│ .  .  .  1  .  .  . │
│ .  .  .  1  .  .  . │
│ .  .  .  2  1  .  . │
│ .  .  .  1  2  .  . │
└───────────────────┘
```

## Deployment

### Deploy to Testnet

```bash
# Generate funded account
stellar keys generate deployer --network <NETWORK> --fund

# Build contract
stellar contract build

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/connect_four.wasm \
  --source deployer \
  --network <NETWORK>
```

### Interact with Deployed Contract

```bash
# Initialize a game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- init_game \
  --player_one <PLAYER_ONE_ADDRESS> \
  --player_two <PLAYER_TWO_ADDRESS>

# Drop a piece in column 3
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- drop_piece \
  --player <PLAYER_ADDRESS> \
  --column 3

# Get game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- get_state

# Check if game is finished
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- is_finished
```

## Game Rules

1. **Objective**: Be the first to connect 4 pieces of your color in a row
2. **Turn Order**: Player One always goes first
3. **Placement**: 
   - Choose a column (0-6)
   - Piece automatically falls to the lowest empty row in that column
   - Cannot place in a full column
4. **Winning**: Connect 4 pieces horizontally, vertically, or diagonally
5. **Draw**: If all 42 spaces are filled with no winner
6. **Invalid Moves**: 
   - Playing out of turn
   - Choosing an out-of-bounds column
   - Choosing a full column
   - Playing after game is over

## Strategy Tips

- **Center Control**: Columns 3 and 4 offer the most winning opportunities
- **Blocking**: Watch for opponent's 3-in-a-row patterns
- **Setup Moves**: Create multiple threats simultaneously
- **Gravity Awareness**: Remember pieces stack from bottom to top

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/cli)
- [Connect Four Wikipedia](https://en.wikipedia.org/wiki/Connect_Four)
