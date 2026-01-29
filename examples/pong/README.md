# Pong On-Chain Game

A fully functional Pong game implemented as a Soroban smart contract on the Stellar blockchain, demonstrating the use of the **Cougr-Core** ECS (Entity Component System) framework for on-chain gaming.

## Overview

This example showcases how to build on-chain game logic using Soroban smart contracts. The implementation includes:
- Complete Pong game mechanics (paddles, ball physics, collisions, scoring)
- On-chain state persistence using Soroban's storage
- Comprehensive test suite with 16 unit tests
- Clean, well-documented code following Rust and Soroban best practices

## Features

- ✅ **Two-player gameplay**: Control paddles for Player 1 and Player 2
- ✅ **Physics simulation**: Ball movement, velocity, and collision detection
- ✅ **Scoring system**: Track scores and determine winners
- ✅ **Boundary detection**: Paddles constrained to field, ball bounces off walls
- ✅ **Game state management**: Initialize, play, and reset games
- ✅ **Fully tested**: 16 comprehensive unit tests covering all game logic

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.89.0 or newer): [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo**: Comes with Rust
- **WASM target**: `rustup target add wasm32-unknown-unknown`
- **Stellar CLI** (optional, for deployment): `cargo install stellar-cli`

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/salazarsebas/Cougr.git
   cd Cougr/examples/pong
   ```

2. **Install WASM target** (if not already installed):
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Building

### Build for Testing
```bash
cargo build
```

### Build WASM Contract
```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM file will be located at:
```
target/wasm32-unknown-unknown/release/pong.wasm
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

**Test Coverage**:
- Game initialization
- Paddle movement (up/down for both players)
- Paddle boundary constraints
- Ball physics and movement
- Wall collisions and bouncing
- Paddle-ball collisions
- Scoring logic (both players)
- Win condition (first to 5 points)
- Game state persistence
- Inactive game state handling

All 16 tests should pass:
```
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Usage

### Contract Functions

#### `init_game() -> GameState`
Initialize a new game with default settings.

**Returns**: Initial game state with centered paddles and ball.

#### `move_paddle(player: u32, direction: i32) -> GameState`
Move a player's paddle.

**Parameters**:
- `player`: Player ID (1 = Player1, 2 = Player2)
- `direction`: Movement direction (-1 = Up, 1 = Down)

**Returns**: Updated game state.

#### `update_tick() -> GameState`
Simulate one game tick (physics update).

**Returns**: Updated game state after physics simulation.

**Game Logic**:
- Updates ball position based on velocity
- Detects and handles wall collisions
- Detects and handles paddle collisions
- Awards points when ball passes paddles
- Ends game when a player reaches winning score (5 points)

#### `get_game_state() -> GameState`
Retrieve the current game state.

**Returns**: Current game state.

#### `reset_game() -> GameState`
Reset the game to initial state.

**Returns**: Fresh game state.

### Game State Structure

```rust
pub struct GameState {
    pub player1_paddle_y: i32,    // Player 1 paddle Y position
    pub player2_paddle_y: i32,    // Player 2 paddle Y position
    pub ball_x: i32,              // Ball X position
    pub ball_y: i32,              // Ball Y position
    pub ball_vx: i32,             // Ball X velocity
    pub ball_vy: i32,             // Ball Y velocity
    pub player1_score: u32,       // Player 1 score
    pub player2_score: u32,       // Player 2 score
    pub game_active: bool,        // Whether game is active
}
```

### Game Constants

- **Field Size**: 100 x 60
- **Paddle Height**: 15
- **Paddle Speed**: 2 units per move
- **Ball Speed**: 1 unit per tick
- **Winning Score**: 5 points

## Deployment

### Deploy to Stellar Testnet

> **Note**: Deployment requires the Stellar CLI and a funded test account.

1. **Get a test account** from [Stellar Testnet Faucet](https://faucet-stellar.acachete.xyz)

2. **Deploy the contract**:
   ```bash
   stellar contract deploy \
     --wasm target/wasm32-unknown-unknown/release/pong.wasm \
     --source <your-secret-key> \
     --network testnet
   ```

3. **Save the contract ID** returned from the deployment.

### Invoke Contract Functions

```bash
# Initialize a new game
stellar contract invoke \
  --id <contract-id> \
  --network testnet \
  -- init_game

# Move Player 1's paddle up
stellar contract invoke \
  --id <contract-id> \
  --network testnet \
  -- move_paddle --player 1 --direction -1

# Update game tick
stellar contract invoke \
  --id <contract-id> \
  --network testnet \
  -- update_tick

# Get current game state
stellar contract invoke \
  --id <contract-id> \
  --network testnet \
  -- get_game_state
```

### Deployment Results

**✅ Successfully Deployed to Stellar Testnet**

**Contract ID**: `CADGGDYDBRVRPPG27IZYZJTFUZ47IJFW3QQ5O67QDMZ7UV3VWCZHPYI3`

**Explorer Link**: [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CADGGDYDBRVRPPG27IZYZJTFUZ47IJFW3QQ5O67QDMZ7UV3VWCZHPYI3)

**Test Account**: `GA5VOXGSGDQBIY7W2UJ2GD23V3566NA7OF4YIL4QCFAVM3PGN7QQQHZA`

**Test Invocations**:

1. **Initialize Game**:
   ```bash
   stellar contract invoke --id CADGGDYDBRVRPPG27IZYZJTFUZ47IJFW3QQ5O67QDMZ7UV3VWCZHPYI3 --source pong-test --network testnet -- init_game
   ```
   **Result**: ✅ Success
   ```json
   {
     "ball_vx": 1,
     "ball_vy": 1,
     "ball_x": 50,
     "ball_y": 30,
     "game_active": true,
     "player1_paddle_y": 30,
     "player1_score": 0,
     "player2_paddle_y": 30,
     "player2_score": 0
   }
   ```

2. **Move Paddle** (Player 1 up):
   ```bash
   stellar contract invoke --id CADGGDYDBRVRPPG27IZYZJTFUZ47IJFW3QQ5O67QDMZ7UV3VWCZHPYI3 --source pong-test --network testnet -- move_paddle --player 1 --direction -1
   ```
   **Result**: ✅ Success - Paddle moved from y=30 to y=28
   ```json
   {
     "player1_paddle_y": 28,
     "player2_paddle_y": 30,
     ...
   }
   ```

3. **Update Tick** (Physics simulation):
   ```bash
   stellar contract invoke --id CADGGDYDBRVRPPG27IZYZJTFUZ47IJFW3QQ5O67QDMZ7UV3VWCZHPYI3 --source pong-test --network testnet -- update_tick
   ```
   **Result**: ✅ Success - Ball moved from (50,30) to (51,31)
   ```json
   {
     "ball_x": 51,
     "ball_y": 31,
     ...
   }
   ```

**Deployment Date**: January 23, 2026

**Transaction Hashes**:
- Deploy: `acd8c82bb0d7167fdd7b438af49dc78e47a90ed9fa682574d20e621aa01769a3`
- [View on Stellar Expert](https://stellar.expert/explorer/testnet/tx/acd8c82bb0d7167fdd7b438af49dc78e47a90ed9fa682574d20e621aa01769a3)

## Architecture

This contract demonstrates the **Cougr-Core ECS (Entity Component System) pattern** adapted for Soroban smart contracts:

### ECS Components

The game state is organized using Cougr-Core's component pattern:

- **PaddleComponent**: Represents paddle entities with player ID and Y position
- **BallComponent**: Represents the ball entity with position (x, y) and velocity (vx, vy)
- **ScoreComponent**: Represents the score entity with both players' scores and game status

```rust
#[contracttype]
pub struct PaddleComponent {
    pub player_id: u32,
    pub y_position: i32,
}

#[contracttype]
pub struct BallComponent {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
}

#[contracttype]
pub struct ScoreComponent {
    pub player1_score: u32,
    pub player2_score: u32,
    pub game_active: bool,
}
```

### ECS Systems

Game logic is organized into systems following Cougr-Core's system pattern:

1. **PhysicsSystem**: Updates ball position based on velocity
   ```rust
   fn physics_system(world: &mut ECSWorldState) {
       world.ball.x += world.ball.vx;
       world.ball.y += world.ball.vy;
   }
   ```

2. **CollisionSystem**: Handles wall and paddle collision detection
   - Detects wall bounces (top/bottom)
   - Detects paddle hits (left/right)
   - Updates ball velocity on collision

3. **ScoringSystem**: Manages scoring and win conditions
   - Awards points when ball passes paddles
   - Checks for game over (first to 5 points)
   - Resets ball position after scoring

### Storage Strategy

- **ECSWorldState**: Serializable structure containing all game components
- **Soroban Instance Storage**: Persists the ECS world between contract invocations
- **Component Pattern**: Each game object (paddles, ball, score) is represented as a component
- **System Pattern**: Game logic is organized into discrete, reusable systems

### Why This Approach?

This implementation demonstrates how Cougr-Core's ECS architecture can be adapted for blockchain constraints:

1. **Component Organization**: Clear separation of data (components) from logic (systems)
2. **Scalability**: Easy to add new components or systems for game features
3. **Testability**: Systems can be tested independently
4. **Soroban Compatibility**: ECS patterns adapted to work with Soroban's storage model

For more complex games with many entities, the full Cougr-Core ECS framework provides additional features like entity queries, component archetypes, and parallel system execution.

## Code Structure

```
examples/pong/
├── Cargo.toml          # Dependencies and build configuration
├── README.md           # This file
└── src/
    ├── lib.rs          # Main contract implementation
    └── test.rs         # Comprehensive test suite
```

## Troubleshooting

### Build Errors

**Error**: `can't find crate for 'core'`
```bash
rustup target add wasm32-unknown-unknown
```

**Error**: `Rust version too old`
```bash
rustup update
```

### Test Failures

If tests fail, ensure you're using the correct Rust version:
```bash
rustc --version  # Should be 1.89.0 or newer
```

### Deployment Issues

**Network errors**: Use `--simulate` flag first to test without deploying:
```bash
stellar contract invoke --id <contract-id> --network testnet --simulate -- init_game
```

**Insufficient funds**: Get more test XLM from the [faucet](https://faucet-stellar.acachete.xyz)

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)

## License

Licensed under MIT OR Apache-2.0

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Documentation](https://www.rust-lang.org/learn)

## Acknowledgments

This example was created to demonstrate on-chain gaming capabilities using Soroban smart contracts and serves as a practical guide for developers building games on the Stellar blockchain.
