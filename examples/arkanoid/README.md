# Arkanoid

On-chain Arkanoid game demonstrating ECS patterns (components & systems) on the Stellar blockchain using `Cougr-Core`.

# ECS Architecture
Components

`PaddleComponent` — Stores horizontal paddle position.

`BallComponent` — Tracks ball position and velocity.

`BricksComponent` — Represents the grid of breakable bricks (10×5).

`ScoreComponent` — Holds score, lives, and game active state.

Systems

`physics_system()` — Updates ball position according to velocity.

`collision_system()` — Handles wall, paddle, and brick collisions.

`scoring_system()` — Updates score, tracks remaining bricks, and determines win/loss.

# Build & Test
```bash
# Standard build
cargo build

# Release build
cargo build --release

# WASM build for Soroban
cargo build --target wasm32-unknown-unknown --release

# Build using Stellar CLI
stellar contract build

# Run all tests
cargo test
```

## Deploy to Testnet
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/arkanoid.wasm \
  --source <YOUR_TESTNET_KEY> \
  --network testnet
```

# Contract API

| Function          | Parameters       | Description                                     |
| ----------------- | ---------------- | ----------------------------------------------- |
| `init_game`       | —                | Initializes ECS world and game state.           |
| `move_paddle`     | `direction: i32` | Moves paddle left (-1) or right (+1).           |
| `update_tick`     | —                | Advances one game frame (physics & collisions). |
| `get_game_state`  | —                | Returns current game state.                     |
| `check_game_over` | —                | Returns `true` if game is over.                 |

# Game Flow

Initialize Game: `Call init_game()`.

Play Loop:

 Call `move_paddle(direction)` to control the paddle.

 Call `update_tick()` to advance ball, check collisions, and update score/lives.

 Call `get_game_state()` to read positions, score, and remaining bricks.

Game Over: When lives reach 0 or all bricks are broken.

# Resources
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Guide](https://developers.stellar.org/docs/tools/cli)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
