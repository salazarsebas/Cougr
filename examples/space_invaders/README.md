
# Space Invaders On-Chain Game

# Space Invaders - On-Chain Game Example


> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

## Purpose and pattern


This example demonstrates a shooter entity-update loop on Soroban with Cougr ECS concepts. It remains transitional while the arcade examples converge on the canonical `snake` `GameApp` architecture.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | `none` | `()` | Initializes ship, invaders, score, lives, and game flags. |
| `move_ship` | `direction: i32` | `i32` | Moves the ship left/right within bounds and returns x position. |
| `shoot` | `none` | `bool` | Spawns a player bullet when possible. |
| `update_tick` | `none` | `bool` | Advances bullets, invaders, collisions, score, and game-over checks. |
| `get_score` | `none` | `u32` | Returns score. |
| `get_lives` | `none` | `u32` | Returns remaining lives. |
| `get_ship_position` | `none` | `i32` | Returns ship x position. |
| `check_game_over` | `none` | `bool` | Returns terminal state. |
| `get_active_invaders` | `none` | `u32` | Returns number of active invaders. |
| `get_entity_count` | `none` | `u32` | Returns total tracked entity count. |

## Live Deployment

| Network | Contract ID | Status |
|---------|-------------|--------|
| **Testnet** | [`<CONTRACT_ID>`](https://stellar.expert/explorer/testnet/contract/<CONTRACT_ID>) | 🟢 Active |


## Architecture overview


```text
contract entrypoint
  ├─ reads game state from Soroban storage
  ├─ applies input or tick systems
  └─ writes updated state back to storage

## Overview

This example demonstrates how to build on-chain game logic on the Stellar blockchain using **cougr-core's ECS architecture**. The game focuses exclusively on smart contract logic (no graphical interface) and includes:

| Feature | Description |
|---------|-------------|
| 🚀 **Ship Control** | Left/right movement with bounds checking |
| 👾 **Invader Grid** | 4×8 formation with wave-based movement |
| 💥 **Bullet System** | Player and enemy projectiles with velocity |
| 🎯 **Collision Detection** | Position-based hit detection |
| ❤️ **Health System** | Lives tracking using Health components |
| 🏆 **Scoring** | Point-based scoring by invader type |

---

## Why Cougr-Core?

**Cougr-Core** provides an ECS (Entity-Component-System) architecture specifically designed for Soroban smart contracts. Here's how it benefits this project:

### Benefits of Using Cougr-Core

| Benefit | Description | Example in This Project |
|---------|-------------|------------------------|
| **Modular Components** | Reusable data structures attached to entities | `EntityPosition`, `Velocity`, `Health` used by Ship, Invaders, and Bullets |
| **Separation of Concerns** | Logic (Systems) separated from data (Components) | Movement System updates all entities with Velocity |
| **Type Safety** | Rust's type system prevents component misuse | `CougrPosition` ensures consistent coordinate handling |
| **WASM Optimization** | ECS optimizes memory access patterns for WASM | Efficient iteration over entity components |
| **Scalability** | Easy to add new features without refactoring | Adding new entity types only requires new components |
| **On-Chain Ready** | Designed for blockchain state persistence | Components serialize to Soroban storage |

### ECS Architecture in Practice

```rust
// Using cougr-core's Position component
use cougr_core::Position as CougrPosition;

// Entity with Position, Velocity, and Health components
pub struct Bullet {
    pub position: EntityPosition,   // Where the bullet is
    pub velocity: Velocity,         // How it moves
    pub active: bool,               // Entity state
}

// Movement System: Apply velocity to position
impl Bullet {
    pub fn update(&mut self) {
        self.velocity.apply_to(&mut self.position);
    }
}

```

Ship, invader, bullet, score, and lives state is currently compacted in contract storage. `components.rs` and `systems.rs` expose the transitional ECS boundary for future migration.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | Per-contract game state where used by this example. | Keeps small arcade state close to the contract instance. |
| Persistent storage | Player- or world-scoped state where the example needs durable keyed state. | Keeps game progress available across invocations. |
| Temporary storage | Not used. | The examples favor deterministic recalculation over ephemeral caches. |


## Main gameplay flow

## ️ Quick Start


1. Call the initialization function to create the starting state.
2. Submit an input action such as movement, jump, flap, rotation, or shoot.
3. Call the tick/update function to run deterministic simulation logic.
4. Query public getters for score, position, active state, or terminal status.
5. Stop when the game-over/completed condition is reached, or reset/reinitialize where supported.

## Cougr APIs used

- `ComponentTrait` and custom component modules document the ECS data boundary.
- `SimpleWorld`, `SimpleQueryBuilder`, `GameApp`, `ScheduleStage`, or `SystemConfig` are used where this transitional example has already adopted the maintained runtime shape.
- Auth, privacy, ZK, and standards APIs are intentionally not used; these arcade examples focus on deterministic game logic.

## Build and test commands

```bash
cargo test

stellar contract build
```

## Known limitations

```

**Test Results**: 13 tests passing ✅

| Test | Description |
|------|-------------|
| `test_init_game` | Game initializes with correct defaults |
| `test_move_ship_left/right` | Ship movement works correctly |
| `test_move_ship_left/right_bounds` | Ship respects boundaries |
| `test_shoot` | Shooting creates bullets |
| `test_shoot_cooldown` | Cooldown prevents rapid fire |
| `test_shoot_after_cooldown` | Shooting works after cooldown |
| `test_update_tick` | Game loop advances correctly |
| `test_score_increase` | Score increases on hits |
| `test_invader_destruction` | Invaders can be destroyed |
| `test_game_over_no_lives` | Game ends when lives = 0 |
| `test_no_move_when_game_over` | No actions after game over |

---

## Contract API

### Core Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `init_game` | - | - | Initialize new game with ECS World |
| `move_ship` | `direction: i32` | `i32` | Move ship (-1=left, 1=right) |
| `shoot` | - | `bool` | Fire bullet (true if successful) |
| `update_tick` | - | `bool` | Advance game (true if running) |

### Query Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `get_score` | `u32` | Current player score |
| `get_lives` | `u32` | Remaining lives |
| `get_ship_position` | `i32` | Ship X coordinate |
| `check_game_over` | `bool` | Game over status |
| `get_active_invaders` | `u32` | Remaining invader count |
| `get_entity_count` | `u32` | Cougr-core entity count |

---

## Game Mechanics

### Invaders

| Type | Position | Points | Health |
|------|----------|--------|--------|
| 🦑 Squid | Top row | 30 pts | 1 HP |
| 🦀 Crab | Middle rows | 20 pts | 1 HP |
| 🐙 Octopus | Bottom row | 10 pts | 1 HP |

**Behavior**:
- Move horizontally in formation
- Descend when reaching screen edge
- Game over if they reach player's row

### Player Ship

| Property | Value |
|----------|-------|
| Starting Lives | 3 |
| Position | Center of game board |
| Movement | Left/Right within bounds |
| Shoot Cooldown | 3 ticks |

### Game Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `GAME_WIDTH` | 40 | Board width |
| `GAME_HEIGHT` | 30 | Board height |
| `INVADER_COLS` | 8 | Invaders per row |
| `INVADER_ROWS` | 4 | Invader rows |
| `BULLET_SPEED` | 2 | Positions per tick |

---

## Deploy to Testnet

### 1. Setup Identity

```bash
# Generate a new identity
stellar keys generate --global deployer --network <NETWORK>

# Fund the account
stellar keys address deployer | xargs -I {} curl "https://friendbot.stellar.org?addr={}"
```

### 2. Build & Deploy

```bash
# Build WASM
stellar contract build

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/space_invaders.wasm \
  --source deployer \
  --network <NETWORK>
```

### 3. Initialize & Play

```bash
# Set your contract ID
CONTRACT_ID="your_contract_id_here"

# Initialize game
stellar contract invoke --id $CONTRACT_ID --source deployer --network <NETWORK> -- init_game

# Play!
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> -- move_ship --direction 1
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> -- shoot
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> -- update_tick
stellar contract invoke --id $CONTRACT_ID --network <NETWORK> -- get_score
```

---

## Project Structure

```
examples/space_invaders/
├── Cargo.toml          # Dependencies including cougr-core
├── README.md           # This documentation
└── src/
    ├── lib.rs          # Contract entry points & ECS systems
    ├── game_state.rs   # ECS Components (Position, Velocity, Health)
    └── test.rs         # Unit tests (13 tests)
```

---

## License


- Transitional code may preserve older storage or scheduling patterns for compatibility reference.
- No authentication, matchmaking, real-time rendering, or production randomness is included.
- One contract instance generally represents one game or one keyed set of player games.
- For new work, prefer the canonical `snake` module split and `GameApp` tick wiring.
