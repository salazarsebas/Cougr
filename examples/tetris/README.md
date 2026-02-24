# Tetris

Classic Tetris game implemented on-chain using the cougr-core ECS framework on the Stellar blockchain via Soroban.
Demonstrates entity-component-system patterns for piece movement, rotation, collision detection, line clearing, scoring, and level progression.

# ECS Architecture
Components

`Piece` — Active tetromino, including shape, X/Y position, and rotation

`GameState` — Stores board state (20x10), current and next pieces, score, level, lines cleared, and game over status

`TetrominoShape` — Enum representing the 7 standard Tetris pieces (I, J, L, O, S, T, Z)

Systems

`Movement System` — Moves current piece left, right, down, or hard drop

`Rotation System` — Rotates pieces clockwise with collision checks

`Gravity System` — Moves pieces downward each tick automatically

`Collision System` — Detects piece collisions with walls, floor, or existing blocks

`Line Clearing System` — Removes completed lines and updates score/level

`Game Logic` — Spawns new pieces, checks for game over, and updates board state

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/tetris.wasm --source <secret-key> --network testnet
```

# Contract API
| Function      | Parameters | Description                                                                             |
| ------------- | ---------- | --------------------------------------------------------------------------------------- |
| `init_game`   | `env`      | Initializes the game, creates empty board, and spawns initial pieces                    |
| `move_left`   | `env`      | Moves current piece left if possible; returns success boolean                           |
| `move_right`  | `env`      | Moves current piece right if possible; returns success boolean                          |
| `move_down`   | `env`      | Moves current piece down (soft drop); locks piece if movement fails                     |
| `rotate`      | `env`      | Rotates current piece clockwise; returns success boolean                                |
| `drop`        | `env`      | Hard drops current piece until it locks; returns number of rows dropped                 |
| `update_tick` | `env`      | Advances game tick with gravity; locks piece if it cannot move down                     |
| `get_state`   | `env`      | Returns current `GameState` including board, pieces, score, level, and game over status |

