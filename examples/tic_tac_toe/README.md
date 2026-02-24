# Tic Tac Toe

A classic 3x3 Tic Tac Toe game implemented on-chain using `cougr-core` ECS patterns with Soroban.
Supports two players, turn-based gameplay, win/draw detection, and on-chain move validation.

# ECS Architecture
Components

`BoardComponent` — Stores the 3x3 board (0 = Empty, 1 = X, 2 = O) and associated entity ID

`PlayerComponent` — Stores both players' addresses (X and O) and entity ID

`GameStateComponent` — Tracks current turn, move count, and game status (0 = InProgress, 1 = XWins, 2 = OWins, 3 = Draw)

`ECSWorldState` — Aggregates board, players, and game state for storage

Systems

`Validation System` — Ensures moves are legal (correct player, unoccupied cell, game in progress)

`Execution System` — Updates board with player's move and increments move count

`Win Detection System` — Checks for winning patterns or draw conditions

`Turn System` — Switches active player after each valid move

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/tictactoe.wasm --source <secret-key> --network testnet
```

# Contract API
| Function        | Parameters                    | Description                                                                                               |
| --------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------- |
| `init_game`     | `env`, `player_x`, `player_o` | Initializes a new game with two players and an empty board                                                |
| `make_move`     | `env`, `player`, `position`   | Makes a move at `position` (0–8). Returns `MoveResult` with success flag, updated game state, and message |
| `get_state`     | `env`                         | Returns current `GameState` including board, players, turn, move count, and status                        |
| `is_valid_move` | `env`, `position`             | Checks if a move is valid (empty cell, game in progress)                                                  |
| `get_winner`    | `env`                         | Returns winner address if game is over, otherwise `None`                                                  |
| `reset_game`    | `env`                         | Resets the game with the same players and empty board                                                     |

