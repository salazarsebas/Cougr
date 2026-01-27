# Flappy Bird - On-Chain Game Example using Cougr-Core

This example demonstrates how to build an on-chain game using the `cougr-core` ECS (Entity Component System) framework on the Stellar blockchain via Soroban smart contracts.

## Overview

This is a fully functional Flappy Bird game implemented as a Soroban smart contract. The game demonstrates:

- **ECS Architecture**: Using cougr-core's Entity Component System for game logic
- **On-Chain State Management**: Storing game state persistently on the blockchain
- **System-Based Logic**: Implementing game mechanics as modular systems
- **Component Serialization**: Proper serialization of game components for blockchain storage

## Learning Objectives

By studying this example, you'll learn:

1. How to integrate `cougr-core` into a Soroban smart contract
2. How to serialize and deserialize ECS World state for on-chain storage
3. How to implement game systems (gravity, movement, collision detection)
4. How to create custom components with proper serialization
5. How to structure a turn-based on-chain game

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (1.70.0 or later): [Install Rust](https://www.rust-lang.org/tools/install)
- **Stellar CLI**: [Install Stellar CLI](https://developers.stellar.org/docs/tools/cli/install)
- **wasm32-unknown-unknown** target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

## Project Structure

```
examples/flappy_bird/
├── Cargo.toml              # Project dependencies and configuration
├── src/
│   ├── lib.rs              # Main contract implementation
│   ├── components.rs       # Custom game components (BirdState, PipeConfig)
│   ├── systems.rs          # Game systems (gravity, collision, scoring)
│   └── world_storage.rs    # World serialization helpers
└── README.md               # This file
```

## Architecture

### Components

The game uses the following components from the ECS pattern:

1. **Position** (from cougr-core): 2D coordinates (x, y)
2. **Velocity** (from cougr-core): Movement vector (x, y)
3. **BirdState**: Tracks whether the bird is alive
4. **PipeConfig**: Stores pipe gap size and position
5. **PipeMarker**: Identifies entities as pipes and tracks if bird passed them

### Systems

Game logic is organized into systems:

1. **Gravity System**: Applies downward acceleration to the bird
2. **Movement System**: Updates positions based on velocities
3. **Pipe Movement System**: Moves pipes horizontally
4. **Collision System**: Detects bird-pipe and bird-ground collisions
5. **Score System**: Increments score when bird passes pipes

### Contract Functions

- `init_game()`: Initialize a new game with bird and pipes
- `flap()`: Make the bird jump (apply upward velocity)
- `update_tick()`: Execute one game tick (run all systems)
- `get_score() -> u32`: Get current score
- `check_game_over() -> bool`: Check if game has ended
- `get_bird_pos() -> (i32, i32)`: Get bird's current position

## Building the Contract

### 1. Build with Cargo

```bash
cd examples/flappy_bird
cargo build --target wasm32-unknown-unknown --release
```

### 2. Build with Stellar CLI

```bash
stellar contract build
```

This will generate the WASM file at:
```
target/wasm32-unknown-unknown/release/flappy_bird.wasm
```

### 3. Optimize (Optional)

For production, you can further optimize the WASM:

```bash
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/flappy_bird.wasm
```

## Running Tests

Run the comprehensive test suite:

```bash
cargo test
```

The tests cover:
- Game initialization
- Bird flapping mechanics
- Gravity and physics
- Collision detection
- Score tracking
- Game over conditions

## Deploying to Testnet

### 1. Create and Fund Account

Generate a new keypair:

```bash
stellar keys generate test-account --network testnet
```

Get the account address:

```bash
stellar keys address test-account
```

Fund the account using Friendbot:

```bash
curl "https://friendbot.stellar.org?addr=$(stellar keys address test-account)"
```

### 2. Deploy Contract

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flappy_bird.wasm \
  --source test-account \
  --network testnet
```

Save the returned CONTRACT_ID for later use.

### 3. Play the Game

#### Initialize Game

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- init_game
```

#### Make Bird Flap

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- flap
```

#### Advance Game Tick

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- update_tick
```

#### Check Score

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- get_score
```

#### Check Game Status

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- check_game_over
```

#### Get Bird Position

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network testnet \
  -- get_bird_pos
```

## Game Mechanics

### Constants

- `GRAVITY`: 2 pixels per tick
- `FLAP_VELOCITY`: -15 pixels per tick (upward)
- `PIPE_SPEED`: 3 pixels per tick (leftward)
- `GROUND_Y`: 400 pixels
- `BIRD_SIZE`: 20 pixels (hitbox radius)
- `PIPE_GAP`: 100 pixels

### How to Play

1. Initialize a new game with `init_game()`
2. Make the bird flap with `flap()` to gain upward velocity
3. Advance the game with `update_tick()` - this applies:
   - Gravity to the bird
   - Movement to all entities
   - Collision detection
   - Score updates
4. Repeat steps 2-3 to keep the bird alive and score points
5. Game ends when bird hits ground, ceiling, or pipes

### Scoring

- Score increases by 1 for each pipe successfully passed
- Bird must pass through the gap without touching the pipe

## Code Examples

### Creating a Custom Component

```rust
#[contracttype]
#[derive(Clone, Debug)]
pub struct BirdState {
    pub is_alive: bool,
}

impl ComponentTrait for BirdState {
    fn component_type() -> Symbol {
        symbol_short!("birdstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let value: u8 = if self.is_alive { 1 } else { 0 };
        bytes.append(&Bytes::from_array(env, &[value]));
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let is_alive = data.get(0).unwrap() != 0;
        Some(Self { is_alive })
    }
}
```

### Implementing a Game System

```rust
pub fn apply_gravity(world: &mut World, env: &Env) {
    for entity in world.entities.iter_entities() {
        let entity_id = entity.id();
        if entity.has_component(&symbol_short!("birdstate")) {
            if let Some(vel_comp) = world.get_component(entity_id, &symbol_short!("velocity")) {
                if let Some(mut velocity) = Velocity::deserialize(env, vel_comp.data()) {
                    velocity.y += GRAVITY;
                    let new_vel_comp = Component::new(
                        symbol_short!("velocity"),
                        velocity.serialize(env)
                    );
                    world.storage.add_component(entity_id, new_vel_comp);
                }
            }
        }
    }
}
```

## Troubleshooting

### Common Issues

**Issue**: `error: failed to compile`
- **Solution**: Ensure you have the correct Rust version (1.70.0+) and wasm32 target installed
- Run: `rustup update && rustup target add wasm32-unknown-unknown`

**Issue**: `stellar: command not found`
- **Solution**: Install Stellar CLI following the [official guide](https://developers.stellar.org/docs/tools/cli/install)

**Issue**: Contract size too large
- **Solution**: Ensure you're using release mode and the optimizations in Cargo.toml:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  ```

**Issue**: Transaction fails with "budget exceeded"
- **Solution**: On-chain games are resource-intensive. Consider simplifying game logic or reducing the number of entities

### Debugging Tips

1. Use `cargo test` to verify logic before deploying
2. Use `--simulate` flag to test transactions without broadcasting:
   ```bash
   stellar contract invoke --id <CONTRACT_ID> --simulate -- init_game
   ```
3. Check contract logs in Stellar Explorer
4. Use smaller test scenarios to isolate issues

## Performance Considerations

### On-Chain Constraints

- **WASM Binary Size**: Keep contract under 1MB (optimized binary should be ~200KB)
- **Transaction Costs**: Each transaction consumes fees - minimize storage operations
- **Execution Budget**: Soroban has CPU and memory limits - keep game ticks simple

### Optimizations Used

1. **Opt-level "z"**: Optimize for size in release builds
2. **LTO**: Link-time optimization reduces binary size
3. **Minimal Storage**: Only store essential game state
4. **Efficient Queries**: Cache entity lookups when possible

## Next Steps

### Extending the Game

1. **Add Power-ups**: Create new components for temporary abilities
2. **Difficulty Scaling**: Increase pipe speed over time
3. **Multiplayer**: Store multiple game states with player IDs
4. **Leaderboard**: Track high scores across players
5. **Visual Output**: Create a frontend that reads contract state and renders graphics

### Learning More

- [Cougr-Core Documentation](../../README.md)
- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [ECS Pattern](https://en.wikipedia.org/wiki/Entity_component_system)

## License

This example is part of the Cougr project and follows the same license.

## Contributing

Found a bug or have an improvement? Please open an issue or pull request in the main [Cougr repository](https://github.com/salazarsebas/Cougr).
