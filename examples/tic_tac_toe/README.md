# Tic Tac Toe On-Chain Game

A fully functional Tic Tac Toe game implemented as a Soroban smart contract on the Stellar blockchain, demonstrating the **Cougr-Core** ECS (Entity Component System) framework for on-chain gaming.

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

| System                 | Responsibility                                    |
| ---------------------- | ------------------------------------------------- |
| `validation_system`    | Enforces game rules (turn order, valid positions) |
| `execution_system`     | Applies moves to the board                        |
| `win_detection_system` | Checks all 8 winning patterns                     |
| `turn_system`          | Manages turn transitions                          |

## Features

| Feature              | Description                                            |
| -------------------- | ------------------------------------------------------ |
| Two-player gameplay  | Uses Stellar addresses for player identification       |
| Turn-based mechanics | X always goes first, enforced turn order               |
| Win detection        | All 8 patterns (3 rows, 3 columns, 2 diagonals)        |
| Draw detection       | Recognizes full board with no winner                   |
| Move validation      | Rejects invalid positions, occupied cells, wrong turns |
| Game reset           | Restart with same players                              |

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

| Test Category  | Count  | Coverage                                        |
| -------------- | ------ | ----------------------------------------------- |
| Initialization | 2      | Game setup, state retrieval                     |
| Valid moves    | 3      | X moves, O moves, position validation           |
| Invalid moves  | 5      | Wrong turn, occupied, out of bounds, non-player |
| Win conditions | 8      | All rows, columns, diagonals                    |
| Draw           | 2      | Full board, post-draw state                     |
| Game over      | 2      | No moves after win/draw                         |
| Reset          | 2      | Mid-game reset, post-win reset                  |
| State          | 3      | Persistence, move counting, winner retrieval    |
| **Total**      | **33** | **All passing**                                 |

## Contract API

### Functions

| Function        | Parameters                             | Returns           | Description             |
| --------------- | -------------------------------------- | ----------------- | ----------------------- |
| `init_game`     | `player_x: Address, player_o: Address` | `GameState`       | Initialize new game     |
| `make_move`     | `player: Address, position: u32`       | `MoveResult`      | Make a move (0-8)       |
| `get_state`     | -                                      | `GameState`       | Get current state       |
| `is_valid_move` | `position: u32`                        | `bool`            | Check if move is valid  |
| `get_winner`    | -                                      | `Option<Address>` | Get winner's address    |
| `reset_game`    | -                                      | `GameState`       | Reset with same players |

### Board Positions

```text
 0 | 1 | 2
-----------
 3 | 4 | 5
-----------
 6 | 7 | 8
```

### Data Structures

**GameState**
| Field        | Type       | Description                            |
| ------------ | ---------- | -------------------------------------- |
| `cells`      | `Vec<u32>` | Board state (0=Empty, 1=X, 2=O)        |
| `player_x`   | `Address`  | Player X's address                     |
| `player_o`   | `Address`  | Player O's address                     |
| `is_x_turn`  | `bool`     | True if X's turn                       |
| `move_count` | `u32`      | Total moves made                       |
| `status`     | `u32`      | 0=InProgress, 1=XWins, 2=OWins, 3=Draw |

**MoveResult**
| Field        | Type        | Description            |
| ------------ | ----------- | ---------------------- |
| `success`    | `bool`      | Whether move succeeded |
| `game_state` | `GameState` | Updated state          |
| `message`    | `Symbol`    | Status code            |

### Error Messages

| Code       | Meaning                          |
| ---------- | -------------------------------- |
| `ok`       | Move successful                  |
| `invalid`  | Position out of bounds (not 0-8) |
| `occupied` | Cell already has a mark          |
| `notturn`  | Not the player's turn            |
| `notplay`  | Address is not a player          |
| `gameover` | Game has already ended           |

## Architecture

```text
ECSWorldState
├── BoardComponent     (entity_id: 0)
│   └── cells: Vec<u32> [9 cells]
├── PlayerComponent    (entity_id: 1)
│   ├── player_x: Address
│   └── player_o: Address
├── GameStateComponent (entity_id: 2)
│   ├── is_x_turn: bool
│   ├── move_count: u32
│   └── status: u32
└── next_entity_id: u32
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
  --wasm target/tic_tac_toe.wasm \
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
  --player_x <PLAYER_X_ADDRESS> \
  --player_o <PLAYER_O_ADDRESS>

# Make a move
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- make_move \
  --player <PLAYER_ADDRESS> \
  --position 4

# Get game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network <NETWORK> \
  -- get_state
```

## Resources

- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/cli)
