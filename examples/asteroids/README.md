# Asteroids (Soroban ECS Example)

This example demonstrates a complete Asteroids game implementation using **Cougr ECS patterns** on Soroban.

## ECS Architecture

This example showcases full Entity-Component-System design following Cougr-Core patterns:

### Components

All game objects are represented as ECS components implementing `ComponentTrait`:

- **ShipComponent** - Player ship with position (x, y), velocity (vx, vy), and rotation angle
- **AsteroidComponent** - Asteroids with position, velocity, and size (splits on destruction)
- **BulletComponent** - Projectiles with position, velocity, and lifetime counter
- **ScoreComponent** - Game state tracking points and lives

### Systems

Game logic is organized into discrete systems that operate on components:

- **MovementSystem** - Updates positions based on velocities with screen wrapping
- **CollisionSystem** - Detects bullet-asteroid and ship-asteroid collisions
- **ShootingSystem** - Spawns bullet entities from ship position and angle
- **AsteroidSplitSystem** - Splits large/medium asteroids into smaller ones on destruction

### Why ECS?

Compared to vanilla Soroban structs, the ECS approach provides:

1. **Modularity** - Components are reusable across different game types
2. **Clarity** - Systems clearly separate concerns (movement, collision, scoring)
3. **Extensibility** - Adding features (power-ups, enemies) requires only new components/systems
4. **Testability** - Individual systems can be tested in isolation

See `examples/pong/` and `examples/arkanoid/` for similar ECS patterns.

## Project Structure

```text
.
├── src
│   ├── lib.rs      # ECS components, systems, and contract implementation
│   └── test.rs     # Component serialization and gameplay tests
├── Cargo.toml
└── README.md
```

## Setup

1) Install Rust + Cargo:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2) Add the WASM target:

```bash
rustup update
rustup target add wasm32v1-none
```

3) Install the Stellar CLI:

```bash
cargo install stellar-cli --locked
```

## Development

Build and test locally:

```bash
cd examples/asteroids
cargo fmt
cargo build
cargo test
cargo clippy
```

Build Soroban WASM:

```bash
stellar contract build
```

## Deployment

Deploy to Testnet:

```bash
stellar network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

stellar keys generate testnet --network <NETWORK>
stellar keys fund testnet --network <NETWORK>

stellar contract deploy \
  --wasm target/wasm32v1-none/release/asteroids.wasm \
  --source testnet \
  --network <NETWORK>
```

Invoke contract methods:

```bash
# Initialize game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  init_game

# Control ship
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  rotate_ship --delta_steps 1

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  thrust_ship

stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  shoot

# Update game state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  update_tick

# Query state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet \
  --network <NETWORK> \
  -- \
  get_game_state
```

## Tests

The test suite covers:

- Component serialization/deserialization
- Ship rotation and thrust mechanics
- Bullet spawning and lifetime cleanup
- Asteroid splitting on destruction
- Ship-asteroid collision and lives system
- Game over conditions (no lives or no asteroids)

Run tests:

```bash
cargo test
```

Expected output:

```
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Game Mechanics

- **Ship**: Rotates in 8 directions, applies thrust in facing direction
- **Bullets**: Travel in straight lines, despawn after 50 ticks
- **Asteroids**: 
  - Size 3 (large) splits into 2 size 2 asteroids
  - Size 2 (medium) splits into 2 size 1 asteroids
  - Size 1 (small) is destroyed completely
- **Scoring**: +10 points per asteroid hit
- **Lives**: Start with 3, lose 1 on ship-asteroid collision
- **Win**: Destroy all asteroids
- **Lose**: Lives reach 0

## Verification (Feb 22, 2026)

All standard build commands pass:

```bash
cargo build    # ✓ No errors
cargo test     # ✓ 11 tests pass
cargo clippy   # ✓ No warnings
```
