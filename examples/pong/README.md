# Pong On-Chain Game

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

## Purpose and pattern

This example demonstrates a two-player paddle physics loop on Soroban with Cougr ECS concepts. It remains transitional while the arcade examples converge on the canonical `snake` `GameApp` architecture.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | `none` | `GameState` | Initializes paddles, ball, score, and active flag. |
| `move_paddle` | `player: u32, direction: i32` | `GameState` | Moves a paddle and returns state. |
| `update_tick` | `none` | `GameState` | Advances ball physics, collisions, scoring, and win checks. |
| `get_game_state` | `none` | `GameState` | Returns current state. |
| `reset_game` | `none` | `GameState` | Resets the game to the initial state. |

## Architecture overview

```text
contract entrypoint
  ├─ reads game state from Soroban storage
  ├─ applies input or tick systems
  └─ writes updated state back to storage
```

Paddle, ball, and score components are in `components.rs`; paddle and ball systems are in `systems.rs`.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | Per-contract game state where used by this example. | Keeps small arcade state close to the contract instance. |
| Persistent storage | Player- or world-scoped state where the example needs durable keyed state. | Keeps game progress available across invocations. |
| Temporary storage | Not used. | The examples favor deterministic recalculation over ephemeral caches. |

## Main gameplay flow

1. Call the initialization function to create the starting state.
2. Submit an input action such as movement, jump, flap, rotation, or shoot.
3. Call the tick/update function to run deterministic simulation logic.
4. Query public getters for score, position, active state, or terminal status.
5. Stop when the game-over/completed condition is reached, or reset/reinitialize where supported.


## Cougr APIs used

2. **Deploy the contract**:
   ```bash
   stellar contract deploy \
     --wasm target/wasm32v1-none/release/pong.wasm \
     --source <your-secret-key> \
     --network <NETWORK>
   ```


- `ComponentTrait` and custom component modules document the ECS data boundary.
- `SimpleWorld`, `SimpleQueryBuilder`, `GameApp`, `ScheduleStage`, or `SystemConfig` are used where this transitional example has already adopted the maintained runtime shape.
- Auth, privacy, ZK, and standards APIs are intentionally not used; these arcade examples focus on deterministic game logic.

## Build and test commands

```bash

cargo test
stellar contract build

# Initialize a new game
stellar contract invoke \
  --id <contract-id> \
  --network <NETWORK> \
  -- init_game

# Move Player 1's paddle up
stellar contract invoke \
  --id <contract-id> \
  --network <NETWORK> \
  -- move_paddle --player 1 --direction -1

# Update game tick
stellar contract invoke \
  --id <contract-id> \
  --network <NETWORK> \
  -- update_tick

# Get current game state
stellar contract invoke \
  --id <contract-id> \
  --network <NETWORK> \
  -- get_game_state
```

### Deployment Results

**Test Account**: `GA5VOXGSGDQBIY7W2UJ2GD23V3566NA7OF4YIL4QCFAVM3PGN7QQQHZA`

**Test Invocations**:

1. **Initialize Game**:
   ```bash
   stellar contract invoke --id <CONTRACT_ID> --source pong-test --network <NETWORK> -- init_game
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
   stellar contract invoke --id <CONTRACT_ID> --source pong-test --network <NETWORK> -- move_paddle --player 1 --direction -1
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
   stellar contract invoke --id <CONTRACT_ID> --source pong-test --network <NETWORK> -- update_tick
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
- Deploy: `<TRANSACTION_HASH>`
- [View on Stellar Expert](https://stellar.expert/explorer/testnet/tx/<TRANSACTION_HASH>)

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
rustup target add wasm32v1-none
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
stellar contract invoke --id <contract-id> --network <NETWORK> --simulate -- init_game

```

## Known limitations

- Transitional code may preserve older storage or scheduling patterns for compatibility reference.
- No authentication, matchmaking, real-time rendering, or production randomness is included.
- One contract instance generally represents one game or one keyed set of player games.
- For new work, prefer the canonical `snake` module split and `GameApp` tick wiring.
