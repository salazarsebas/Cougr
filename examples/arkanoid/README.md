# Arkanoid On-Chain Game

A practical example demonstrating how to use the `cougr-core` package to implement Arkanoid game logic on the Stellar blockchain via Soroban smart contracts.

## Overview

This example showcases:
- **ECS Architecture**: Component-based design using Cougr-Core patterns
- **On-Chain Game Logic**: Complete Arkanoid implementation without graphical interface
- **Soroban Integration**: Smart contract deployment on Stellar blockchain
- **Comprehensive Testing**: Full test coverage of game mechanics

## Game Components

Following the Cougr-Core ECS pattern:

- **PaddleComponent**: Horizontal paddle position
- **BallComponent**: Ball position and velocity
- **BricksComponent**: Grid of breakable bricks (10x5)
- **ScoreComponent**: Score, lives, and game state

## Prerequisites

### Required Tools

1. **Rust** (1.70.0 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Cargo** (comes with Rust)

3. **Stellar CLI**
   ```bash
   cargo install stellar-cli --locked
   ```

4. **WASM Target**
   ```bash
   rustup target add wasm32-unknown-unknown
   rustup target add wasm32v1-none
   ```

### Verify Installation

```bash
rustc --version
cargo --version
stellar --version
```

## Setup

### 1. Clone Repository

```bash
git clone https://github.com/salazarsebas/Cougr.git
cd Cougr/examples/arkanoid
```

### 2. Install Dependencies

```bash
cargo update
```

## Development

### Build

```bash
# Standard build
cargo build

# Release build
cargo build --release

# WASM build
cargo build --target wasm32-unknown-unknown --release

# Stellar CLI build (recommended for deployment)
stellar contract build
```

### Test

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_init_game
```

### Lint

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features -- -D warnings
```

## Deployment

### 1. Setup Testnet Account

Get test XLM from Friendbot:

```bash
# Generate a new keypair
stellar keys generate alice --network testnet

# Fund the account
stellar keys fund alice --network testnet
```

Or use the web faucet: https://faucet-stellar.acachete.xyz

### 2. Deploy Contract

```bash
# Build the contract
stellar contract build

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/arkanoid.wasm \
  --source alice \
  --network testnet
```

Save the contract ID returned (e.g., `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`).

### 3. Invoke Contract Functions

#### Initialize Game

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- init_game
```

**Expected Output:**
```json
{
  "paddle_x": 50,
  "ball_x": 50,
  "ball_y": 50,
  "ball_vx": 1,
  "ball_vy": -1,
  "score": 0,
  "lives": 3,
  "game_active": true,
  "bricks_remaining": 50
}
```

#### Move Paddle

```bash
# Move right (direction = 1)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- move_paddle --direction 1

# Move left (direction = -1)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- move_paddle --direction -1
```

#### Update Game Tick

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- update_tick
```

This advances the game by one frame:
- Moves the ball
- Checks collisions (walls, paddle, bricks)
- Updates score and lives
- Checks win/loss conditions

#### Get Game State

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- get_game_state
```

#### Check Game Over

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- check_game_over
```

Returns `true` if game is over, `false` otherwise.

## Game Flow

1. **Initialize**: Call `init_game()` to set up the game state
2. **Play Loop**:
   - Call `move_paddle(direction)` to control the paddle
   - Call `update_tick()` to advance the game
   - Call `get_game_state()` to check current state
3. **Game Over**: When lives reach 0 or all bricks are broken

## Game Mechanics

### Constants

- **Field**: 100x60 units
- **Paddle**: Width 15, Speed 3
- **Ball**: Speed 1
- **Bricks**: 10 columns × 5 rows (50 total)
- **Lives**: 3

### Scoring

- Breaking a brick: +10 points
- Win condition: Break all bricks
- Loss condition: Lives reach 0

### Collisions

- **Top/Side Walls**: Ball bounces
- **Bottom Wall**: Lose 1 life, ball resets
- **Paddle**: Ball bounces upward
- **Bricks**: Brick breaks, ball bounces, score increases

## Troubleshooting

### Rust Version Issues

```bash
rustup update stable
rustup default stable
```

### Dependency Resolution Errors

```bash
cargo clean
cargo update
cargo build
```

### WASM Build Failures

Ensure WASM targets are installed:
```bash
rustup target add wasm32-unknown-unknown
rustup target add wasm32v1-none
```

### Stellar CLI Not Found

```bash
cargo install stellar-cli --locked --force
```

### Network Errors

Use `--simulate` flag to test without deploying:
```bash
stellar contract invoke --id <CONTRACT_ID> --simulate -- init_game
```

## Architecture

### Cougr-Core Benefits

This example demonstrates how Cougr-Core simplifies on-chain game development:

1. **Component Pattern**: Modular game data (Paddle, Ball, Bricks, Score)
2. **System Pattern**: Organized logic (Physics, Collision, Scoring systems)
3. **Scalability**: Easy to add features (power-ups, multiple balls)
4. **Clarity**: Clear separation of data and operations
5. **Reusability**: Components can be used across different games

### ECS Systems

- **PhysicsSystem**: Updates ball position based on velocity
- **CollisionSystem**: Handles wall, paddle, and brick collisions
- **ScoringSystem**: Manages score, lives, and win/loss conditions

## Testing

The test suite covers:

- ✅ Game initialization
- ✅ Paddle movement (left, right, bounds)
- ✅ Ball physics
- ✅ Wall collisions (top, sides, bottom)
- ✅ Paddle collision
- ✅ Brick breaking
- ✅ Scoring
- ✅ Lives system
- ✅ Game over conditions
- ✅ Inactive game state

Run tests with:
```bash
cargo test
```

## CI/CD

GitHub Actions workflow automatically:
- Checks code compilation
- Runs formatting checks
- Runs clippy lints
- Executes all tests
- Builds WASM contracts
- Builds with Stellar CLI

## License

MIT OR Apache-2.0

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Guide](https://developers.stellar.org/docs/tools/cli)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
