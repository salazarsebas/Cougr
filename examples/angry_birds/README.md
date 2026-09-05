# Angry Birds on Stellar Soroban

A turn-based physics puzzle implementation using the Cougr ECS framework on Stellar/Soroban. This contract demonstrates complex state transitions, entity relationships, and bounded on-chain computation suitable for blockchain gaming.

## Game Design

Angry Birds is a physics-based puzzle game where players launch projectiles (birds) at structures to eliminate targets (pigs). This implementation focuses on turn-based gameplay with deterministic physics simulation.

### Core Mechanics

- **Turn-based shooting**: Players submit angle and power parameters for each shot
- **Deterministic physics**: Fixed-point mathematics ensure reproducible results
- **Material system**: Different materials (wood, glass, stone) have varying damage resistance
- **Bounded computation**: Maximum 50 simulation steps per shot prevents gas limit issues
- **Win/loss conditions**: Win by destroying all targets, lose by running out of birds

## ECS Architecture

This implementation showcases the Cougr-Core ECS framework with the following components and systems:

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| `PositionComponent` | `x: i32`, `y: i32` | Grid position using fixed-point math (scaled by 1000) |
| `HealthComponent` | `hp: u32`, `max_hp: u32` | Hit points for structures and pigs |
| `MaterialComponent` | `kind: enum {Wood, Glass, Stone}` | Damage resistance multiplier |
| `ProjectileComponent` | `bird_type: enum`, `angle: i32`, `power: i32`, `active: bool` | Shot parameters for trajectory resolution |
| `ScoreComponent` | `points: u32`, `birds_remaining: u32` | Running score tracker |
| `LevelConfigComponent` | `level_id: u32`, `status: enum`, `player: Address` | Level metadata and game state |

### Systems

1. **ShotResolutionSystem** - Resolves projectile trajectory using discrete steps (max 50 iterations)
2. **DamageSystem** - Applies damage based on material resistance (wood < glass < stone)
3. **ScoreSystem** - Calculates points from destroyed entities and remaining birds
4. **WinConditionSystem** - Checks if all pigs are eliminated (win) or no birds remain (loss)

## Technical Constraints

### Fixed-Point Mathematics
- All positions and velocities use fixed-point math scaled by 1000
- Eliminates floating-point inconsistencies across different environments
- Ensures deterministic behavior crucial for blockchain applications

### Bounded Computation
- Maximum 50 simulation steps per shot
- Prevents unbounded loops and gas limit issues
- Grid-based collision detection for efficiency

### Deterministic Behavior
- Same input (angle, power, bird_type) always produces the same output
- Critical for fair gameplay and blockchain verification

## Contract API

### `init_level(env: Env, player: Address, level_id: u32) -> LevelState`
Initializes a new game level with default configuration:
- 3 birds available
- Empty game field
- Score reset to 0
- Game status set to InProgress

### `shoot(env: Env, player: Address, angle: i32, power: i32) -> ShotOutcome`
Executes a shot with specified parameters:
- `angle`: Launch angle in degrees (scaled by 1000)
- `power`: Launch power (scaled by 1000)
- Returns shot outcome (hit, miss, win, loss)
- Validates player authorization and game state

### `get_state(env: Env) -> LevelState`
Returns current game state including:
- Level ID and status
- Current score and birds remaining
- Number of entities in the game

### `reset_level(env: Env, player: Address) -> LevelState`
Resets the current level to initial state:
- Maintains same level ID and player
- Resets score, birds, and game status
- Clears all game entities

## Building and Testing

### Prerequisites
- Rust 1.88+ with wasm32 target
- Stellar CLI for contract deployment
- Git for cloning the repository

### Setup

```bash
# Clone the Cougr repository
git clone https://github.com/salazarsebas/Cougr.git
cd Cougr/examples/angry_birds

# Install Rust targets
rustup target add wasm32v1-none

# Install Stellar CLI
brew install stellar-cli  # macOS
# or follow https://github.com/stellar/stellar-cli for other platforms
```

### Build Commands

```bash
# Standard Rust build
cargo build

# Release build with optimizations
cargo build --release

# Stellar contract build (produces .wasm)
stellar contract build

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_level_initialization

# Run tests with output
cargo test -- --nocapture
```

## Deployment

### Testnet Deployment

```bash
# Set up testnet environment
stellar network testnet

# Deploy contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/angry_birds.wasm \
  --source <your-account> \
  --network <NETWORK>

# Note the contract ID for subsequent interactions
```

### Contract Interaction

```bash
# Initialize a level
stellar contract invoke \
  --id <contract-id> \
  --source <player-account> \
  --network <NETWORK> \
  -- init_level \
  --player <player-address> \
  --level_id 1

# Take a shot (45 degrees, medium power)
stellar contract invoke \
  --id <contract-id> \
  --source <player-account> \
  --network <NETWORK> \
  -- shoot \
  --player <player-address> \
  --angle 45000 \
  --power 500000

# Get game state
stellar contract invoke \
  --id <contract-id> \
  --source <any-account> \
  --network <NETWORK> \
  -- get_state
```

## Game Example

```rust
// Initialize game
let state = AngryBirdsContract::init_level(env, player, 1);
// state: LevelState { level_id: 1, status: InProgress, score: 0, birds_remaining: 3, ... }

// Take first shot
let outcome = AngryBirdsContract::shoot(env, player, 45000, 500000);
// outcome: ShotOutcome::Miss (no entities to hit yet)

// Check state after shot
let state = AngryBirdsContract::get_state(env);
// state: LevelState { ..., birds_remaining: 2, ... }
```

## Design Tradeoffs

### Fixed-Point vs Floating-Point
- **Choice**: Fixed-point mathematics scaled by 1000
- **Rationale**: Ensures deterministic behavior across different execution environments
- **Tradeoff**: Limited precision compared to floating-point, but sufficient for game physics

### Bounded Simulation
- **Choice**: Maximum 50 simulation steps per shot
- **Rationale**: Prevents gas limit issues and ensures predictable execution cost
- **Tradeoff**: May limit complex trajectory calculations, but adequate for turn-based gameplay

### Grid-Based Collisions
- **Choice**: Simple circular collision detection with fixed radius
- **Rationale**: Computationally efficient and easy to verify
- **Tradeoff**: Less precise than continuous collision detection, but sufficient for puzzle gameplay

## Integration with Cougr Framework

This example demonstrates several key benefits of the Cougr ECS framework:

1. **Component Modularity**: Each game aspect (position, health, material) is a separate, reusable component
2. **System Separation**: Game logic is organized into discrete systems with clear responsibilities
3. **Scalability**: Adding new features (power-ups, different bird types) requires minimal code changes
4. **Testability**: Components and systems can be tested independently
5. **Clarity**: The ECS architecture makes game state and logic immediately understandable

## Future Enhancements

Potential extensions that showcase Cougr's flexibility:

- **Multiple bird types** with unique behaviors
- **Power-up system** using additional components
- **Level editor** for creating custom scenarios
- **Multiplayer support** with turn management
- **Leaderboard system** for high scores
- **Visual themes** using component-based rendering

## Contributing

When contributing to this example:

1. Follow the existing ECS patterns and component structure
2. Maintain deterministic behavior for all game logic
3. Add comprehensive tests for new features
4. Update documentation for API changes
5. Ensure all build commands pass without errors

## License

This example is licensed under MIT OR Apache-2.0, consistent with the main Cougr project.
