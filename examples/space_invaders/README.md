# 🎮 Space Invaders - On-Chain Game Example

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Tests](https://img.shields.io/badge/tests-13%20passing-brightgreen)](https://github.com/salazarsebas/Cougr)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-blue)](https://stellar.org)

A fully functional Space Invaders game implemented as a **Soroban smart contract** using the `cougr-core` ECS (Entity-Component-System) framework on the Stellar blockchain.

## 🚀 Live Deployment

| Network | Contract ID | Status |
|---------|-------------|--------|
| **Testnet** | [`CD6EUPL7Z255BTDPOCMQVWQ7CNM4ORP7QEFPPHO6JC63HRGLW6PYQAG7`](https://stellar.expert/explorer/testnet/contract/CD6EUPL7Z255BTDPOCMQVWQ7CNM4ORP7QEFPPHO6JC63HRGLW6PYQAG7) | 🟢 Active |

> 🔗 **Explorer**: [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CD6EUPL7Z255BTDPOCMQVWQ7CNM4ORP7QEFPPHO6JC63HRGLW6PYQAG7)

---

## 📋 Overview

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

## 🔧 Architecture

This example uses Cougr as the core architecture. `SimpleWorld` owns all entity state; systems are plain functions that query and mutate it; the world is persisted to Soroban instance storage alongside a `GameMeta` struct for non-entity data.

### Components (`src/components.rs`)

| Component | Data | Used by |
|-----------|------|---------|
| `Position` | x, y | Ship, invaders, bullets |
| `Velocity` | dx, dy | Bullets |
| `Health` | current HP | Ship |
| `ShipTag` | marker | Player ship entity |
| `InvaderTag` | points, active | Invader entities |
| `BulletTag` | owner (0=player, 1=enemy) | Bullet entities |

### Systems (`src/systems.rs`)

| System | Responsibility |
|--------|---------------|
| `movement_system` | Applies each bullet's `Velocity` to its `Position`; despawns out-of-bounds bullets |
| `collision_system` | Player bullets vs invaders, enemy bullets vs ship; returns (score_gained, ship_hit) |
| `invader_movement_system` | Moves the formation, descends on wall contact, returns game-over flag |
| `enemy_shoot_system` | Spawns an enemy bullet entity each interval |

### Runtime shape

```
init_game()
  └─ SimpleWorld::new()
  └─ spawn ship entity → Position, Health, ShipTag
  └─ spawn 32 invader entities → Position, InvaderTag
  └─ save world + GameMeta to instance storage

update_tick()
  └─ load world from storage
  └─ movement_system   (bullets move)
  └─ collision_system  (hits resolved, entities despawned)
  └─ invader_movement_system
  └─ enemy_shoot_system
  └─ save world + GameMeta back to storage
```

Non-entity state (score, tick, cooldown, invader direction, game-over flag) lives in `GameMeta`, stored separately alongside the world.

## 🏗️ Quick Start

### Prerequisites

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | 1.70.0+ | [rustup.rs](https://rustup.rs) |
| Stellar CLI | Latest | [Stellar Docs](https://developers.stellar.org/docs/tools/cli) |
| WASM Target | - | `rustup target add wasm32v1-none` |

### Build

```bash
# Standard Rust build
cargo build

# Build WASM for Soroban deployment
stellar contract build
```

### Test

```bash
cargo test
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

## 📖 Contract API

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

---

## 🎮 Game Mechanics

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

## 🌐 Deploy to Testnet

### 1. Setup Identity

```bash
# Generate a new identity
stellar keys generate --global deployer --network testnet

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
  --network testnet
```

### 3. Initialize & Play

```bash
# Set your contract ID
CONTRACT_ID="your_contract_id_here"

# Initialize game
stellar contract invoke --id $CONTRACT_ID --source deployer --network testnet -- init_game

# Play!
stellar contract invoke --id $CONTRACT_ID --network testnet -- move_ship --direction 1
stellar contract invoke --id $CONTRACT_ID --network testnet -- shoot
stellar contract invoke --id $CONTRACT_ID --network testnet -- update_tick
stellar contract invoke --id $CONTRACT_ID --network testnet -- get_score
```

---

## 📁 Project Structure

```
examples/space_invaders/
├── Cargo.toml          # Dependencies including cougr-core
├── README.md           # This documentation
└── src/
    ├── lib.rs          # Contract entry points, GameMeta, storage helpers
    ├── components.rs   # Position, Velocity, Health, ShipTag, InvaderTag, BulletTag
    ├── systems.rs      # movement, collision, invader_movement, enemy_shoot systems
    └── test.rs         # Unit tests (13 tests)
```

---

## 📄 License

MIT OR Apache-2.0
