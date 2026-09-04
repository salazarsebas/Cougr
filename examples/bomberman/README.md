# Bomberman On-Chain Game

This example demonstrates how to implement a Bomberman game as a smart contract on the Stellar blockchain using Soroban and cougr-core.

## Overview

This contract implements a simplified version of the classic Bomberman game where players can:
- Initialize a game with a grid
- Spawn and move players around the grid
- Place bombs that have individually configurable blast power
- Trigger **chain reactions** - a bomb overlapped by an explosion detonates instantly in the same tick
- Pick up **power-ups** that improve stats (blast radius, bomb capacity, movement speed)
- Handle collisions and scoring
- Check for game-over conditions

## Setup

### Prerequisites

- Rust (latest stable version)
- Cargo
- Stellar CLI (install with `cargo install --locked stellar-cli`)

### Installation

1. Install Rust from https://rustup.rs/
2. Install Stellar CLI:
   ```bash
   cargo install --locked stellar-cli
   ```

### Building the Contract

```bash
# Build the Rust code
cargo build

# Build the WASM file
stellar contract build
```

### Testing and Validation

```bash
# Formatting check
cargo fmt --check

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# Run unit tests
cargo test

# Build WASM (target wasm32v1-none)
stellar contract build
```

## Game Logic

### ECS Components

| Component | Fields | Notes |
|---|---|---|
| `PlayerComponent` | `id, x, y, lives, bomb_capacity, score, bomb_power, speed` | All player state |
| `BombComponent` | `x, y, timer, power, owner_id` | Power set from `bomb_power` at placement time |
| `ExplosionComponent` | `x, y, timer` | Despawned after `EXPLOSION_DURATION` ticks |
| `GridComponent` | `cells: Vec<CellType>` | Walls, destructible blocks |
| `GameStateComponent` | `current_tick, game_over, winner_id` | Global match state |
| `PowerUpComponent` | `x, y, power_up_type` | `Capacity \| Power \| Speed` |

### Power-ups

Three power-up types are supported and modelled as first-class ECS entities:

| Type | Effect |
|---|---|
| `Capacity` | `bomb_capacity += 1` - player may place one more active bomb |
| `Power` | `bomb_power += 1` - new bombs have a larger blast radius |
| `Speed` | `speed += 1` - reserved for future movement throttling |

Power-ups are spawned at game start (deterministic grid positions) and when a
bomb destroys a destructible block (~25% chance via `(x+y)%4==0`).
Walking over a `PowerUpComponent` entity silently applies the buff and removes
the entity from the world (**PickupSystem**).

### Chain Reactions

The **ChainReactionSystem** runs inside `update_tick`:
1. Bombs whose timer hits `0` are placed in a `detonation_queue`.
2. Each bomb is detonated in order - explosions are spawned immediately.
3. After each detonation, every remaining live bomb is checked for overlap
   with a fresh explosion cell.  A hit bomb is removed and pushed to the back
   of the queue, so it detonates in the same tick - forming a cascade.
4. The process repeats until the queue is empty.

### Systems at a Glance

| System | Where | What it does |
|---|---|---|
| `BombTimerSystem` | `update_tick` | Decrements bomb timers |
| `ChainReactionSystem` | `update_tick` | Cascades detonations within one tick |
| `ExplosionPropagationSystem` | `detonate_bomb` | Spawns explosion entities in 4 directions |
| `PowerUpSpawnSystem` | `init_game` + `detonate_bomb` | Creates `PowerUpComponent` entities |
| `PickupSystem` | `move_player` | Applies buff when player steps on a power-up |

### Contract Functions

- `init_game()`: Initializes the game state
- `move_player(player_id, direction)`: Moves a player in the specified direction
- `place_bomb(player_id)`: Places a bomb at the player's current position
- `update_tick()`: Advances timers, triggers explosions, handles collisions
- `get_score(player_id)`: Returns the current score for a player
- `check_game_over()`: Checks if the game has ended

## Integration with Cougr-Core

This example demonstrates how cougr-core simplifies on-chain game development by providing:

- Persistent storage management for game state
- Transaction validation
- Generic game logic utilities
- Efficient data structures for complex game states

## Deployment

### Prerequisites for Deployment

1. **Test Account**: Fund a test account using the Friendbot: https://faucet-stellar.acachete.xyz
2. **Stellar CLI**: Install with `cargo install --locked stellar-cli`
3. **WASM Build**: Ensure the contract builds successfully

### To Testnet

1. Build the WASM file:
   ```bash
   stellar contract build
   ```

2. Deploy the contract:
   ```bash
   stellar contract deploy \
     --wasm target/wasm32v1-none/release/bomberman.wasm \
     --source <your-secret-key> \
     --network <NETWORK>
   ```

3.  the contract ID for future invocations (e.g., `<CONTRACT_ID>`)

https://stellar.expert/explorer/testnet/account/GAQAXKUQYNBHZYZ2OYISPXDZDP2HV57534VMGARGGIICH2BGNKDTKXOX

### Testing the Contract

Invoke functions to test gameplay:

```bash
# Initialize game
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  init_game

# Move player (directions: 0=up, 1=right, 2=down, 3=left)
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  move_player \
  --player_id 1 \
  --direction 1

# Place bomb
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  place_bomb \
  --player_id 1

# Update game tick (advances timers, processes explosions)
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  update_tick

# Get player score
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  get_score \
  --player_id 1

# Check game status
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network <NETWORK> \
  -- \
  check_game_over
```

### Example Game Sequence

1. **Initialize**: Call `init_game` to set up the world
2. **Spawn Players**: In a full implementation, add player spawning functions
3. **Gameplay Loop**:
   - Move players with `move_player`
   - Place bombs with `place_bomb`
   - Advance game state with `update_tick`
   - Check scores and game status periodically
4. **Game Over**: Monitor `check_game_over` for completion

### Cost Monitoring

Monitor transaction costs during testing:
- **Rent**: Storage costs on Stellar ledger
- **Fees**: Network transaction fees
- **CPU/RAM**: Contract execution costs

Use `--simulate` flag to estimate costs before actual deployment:
```bash
stellar contract invoke --simulate [other flags]
```

## Troubleshooting

### Common Issues

1. **Rust version conflicts**: Update Rust with `rustup update`
2. **Stellar CLI not found**: Ensure it's in your PATH after installation
3. **Compilation errors**: Check that all dependencies are correctly specified in Cargo.toml

### Debug Commands

```bash
# Verbose build
cargo build --verbose

# Check Stellar CLI version
stellar --version

# Simulate contract deployment
stellar contract deploy --wasm target/wasm32v1-none/release/bomberman.wasm --simulate
```

## Architecture

The contract uses the **cougr-core ECS (Entity–Component–System)** pattern for all game state:

- **Entities**: Numeric IDs created by `world.spawn_entity()` / removed by `world.despawn_entity()`
- **Components**: Plain structs implementing `ComponentTrait` (typed `serialize` / `deserialize`)
- **Systems**: Free functions / inline blocks inside contract entry-points that compose over component queries

All state is persisted in a single `SimpleWorld` under `DataKey::World`.
The separation of components and systems makes it straightforward to add new mechanics without touching unrelated code.
