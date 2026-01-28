# Snake On-Chain Game

A fully functional Snake game implemented as a Soroban smart contract using the **cougr-core** ECS (Entity-Component-System) framework on the Stellar blockchain.

## Table of Contents

- [Overview](#overview)
- [Why cougr-core?](#why-cougr-core)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Contract Functions](#contract-functions)
- [Deployment](#deployment)
- [Project Structure](#project-structure)
- [Creating Components](#creating-components)
- [Troubleshooting](#troubleshooting)

---

## Overview

This implementation follows classic Snake game rules:

| Rule | Description |
|------|-------------|
| Movement | Snake moves continuously in current direction |
| Control | Player can change direction (cannot reverse) |
| Growth | Eating food increases snake length and score |
| Game Over | Collision with walls or self ends the game |

---

## Why cougr-core?

The **cougr-core** package provides significant advantages for building on-chain games on Stellar/Soroban:

### Benefits Comparison

| Aspect | Without cougr-core | With cougr-core |
|--------|-------------------|-----------------|
| **Component Serialization** | Manual byte encoding for each type | Standardized `ComponentTrait` with `serialize()`/`deserialize()` |
| **Type Identification** | Custom validation per component | Built-in `component_type()` returns unique `Symbol` |
| **Storage Optimization** | One-size-fits-all approach | `Table` vs `Sparse` storage strategies |
| **Entity Management** | Custom ID tracking | Standardized `EntityId` with generation |
| **Code Reusability** | Write from scratch | Extend proven ECS patterns |

### Key Features Used in This Example

```rust
// 1. Import from cougr-core
use cougr_core::component::{Component, ComponentStorage, ComponentTrait};

// 2. Implement ComponentTrait for type-safe serialization
impl ComponentTrait for Position {
    fn component_type() -> Symbol {
        symbol_short!("position")  // Unique identifier
    }

    fn serialize(&self, env: &Env) -> Bytes { /* ... */ }
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> { /* ... */ }

    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table  // Optimized for dense data
    }
}

// 3. Create Component wrapper for unified storage
let component = Component::new(Position::component_type(), position.serialize(&env));
```

### Storage Strategy Optimization

| Component | Storage | Rationale |
|-----------|---------|-----------|
| `Position` | `Table` | Every entity has one, accessed every tick |
| `Direction` | `Table` | Frequently read and updated |
| `SnakeSegment` | `Table` | Dense access pattern for movement |
| `SnakeHead` | `Sparse` | Only one entity, marker component |
| `Food` | `Sparse` | Single entity at a time |

---

## Architecture

### Entity-Component-System Pattern

| Layer | Elements | Description |
|-------|----------|-------------|
| **Entities** | Snake Head, Segments, Food | Game objects identified by ID |
| **Components** | Position, Direction, Markers | Data attached to entities |
| **Systems** | Movement, Collision, Growth | Logic operating on components |

### Components Reference

| Component | Type | Data | Purpose |
|-----------|------|------|---------|
| `Position` | Data | `x: i32, y: i32` | Grid coordinates |
| `DirectionComponent` | Data | `Direction` enum | Movement direction |
| `SnakeHead` | Marker | - | Identifies head entity |
| `SnakeSegment` | Data | `index: u32` | Body segment order |
| `Food` | Marker | - | Identifies food entity |

### Systems Reference

| System | Input | Output | Description |
|--------|-------|--------|-------------|
| `move_snake` | World, grid_size | Option<Position> | Updates positions, detects wall collision |
| `check_self_collision` | World | bool | Detects snake hitting itself |
| `check_food_collision` | World | Option<EntityId> | Detects food consumption |
| `grow_snake` | World | - | Adds segment at tail |
| `spawn_food` | World, tick | - | Places food at unoccupied cell |
| `update_direction` | World, Direction | bool | Validates and applies direction |

---

## Prerequisites

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | Stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| WASM Target | - | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | Latest | `brew install stellar-cli` (macOS) |

### Verify Installation

```bash
rustc --version    # Should show stable version
stellar --version  # Should show CLI version
```

---

## Quick Start

### 1. Clone and Navigate

```bash
git clone https://github.com/salazarsebas/Cougr.git
cd Cougr/examples/snake
```

### 2. Build

```bash
# Development build
cargo build

# Build WASM contract
stellar contract build
```

### 3. Test

```bash
cargo test
```

**Expected Output:**
```
running 30 tests
test result: ok. 30 passed; 0 failed; 0 ignored
```

### 4. Lint

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

---

## Contract Functions

### Initialization Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `init_game` | - | - | Start with 10×10 grid |
| `init_game_with_size` | `grid_size: i32` | - | Start with custom grid |

### Control Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `change_direction` | `direction: u32` | `bool` | Change movement direction |
| `update_tick` | - | - | Advance game one step |

**Direction Values:**

| Value | Direction | Delta (x, y) |
|-------|-----------|--------------|
| 0 | Up | (0, -1) |
| 1 | Down | (0, +1) |
| 2 | Left | (-1, 0) |
| 3 | Right | (+1, 0) |

### Query Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `get_score` | `u32` | Current score |
| `check_game_over` | `bool` | Game ended status |
| `get_head_pos` | `(i32, i32)` | Head coordinates |
| `get_snake_length` | `u32` | Total length |
| `get_food_pos` | `(i32, i32)` | Food coordinates |
| `get_snake_positions` | `Vec<(i32, i32)>` | All positions |
| `get_grid_size` | `i32` | Grid dimensions |

---

## Deployment

### Testnet Deployment

```bash
# 1. Generate keypair
stellar keys generate --global alice --network testnet
stellar keys address alice

# 2. Fund account (visit URL with your address)
# https://friendbot.stellar.org/?addr=<YOUR_ADDRESS>

# 3. Deploy contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/snake.wasm \
  --source alice \
  --network testnet

# Save the returned Contract ID!
```

### Playing the Game

```bash
CONTRACT_ID="<your-contract-id>"

# Initialize
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- init_game

# Change direction (0=Up, 1=Down, 2=Left, 3=Right)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- change_direction --direction 0

# Advance game
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- update_tick

# Check score
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- get_score
```

### Deployed Contract

| Network | Contract ID | Explorer |
|---------|-------------|----------|
| Testnet | `CCMDAHIKL3K5YHBMFYMMP65F6NRTQXICQSJJ2AF7JG7RVRVWGZY2S5LJ` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCMDAHIKL3K5YHBMFYMMP65F6NRTQXICQSJJ2AF7JG7RVRVWGZY2S5LJ) |

---

## Project Structure

```
examples/snake/
├── Cargo.toml              # Dependencies (cougr-core, soroban-sdk)
├── README.md               # This documentation
├── .gitignore              # Ignore rules (test_snapshots/, target/)
└── src/
    ├── lib.rs              # Contract entry points (11 functions)
    ├── components.rs       # Components using cougr-core::ComponentTrait
    ├── systems.rs          # Game logic systems
    └── simple_world.rs     # Entity-component storage
```

| File | Purpose |
|------|---------|
| `lib.rs` | Soroban contract with public functions and tests |
| `components.rs` | Component definitions implementing `ComponentTrait` |
| `systems.rs` | Game mechanics (movement, collision, spawning) |
| `simple_world.rs` | Entity and component storage layer |

---

## Creating Components

### Using cougr-core's ComponentTrait

```rust
use cougr_core::component::{Component, ComponentStorage, ComponentTrait};
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

pub struct MyComponent {
    pub value: u32,
}

impl ComponentTrait for MyComponent {
    // Unique identifier for this component type
    fn component_type() -> Symbol {
        symbol_short!("mycomp")
    }

    // Serialize to bytes for on-chain storage
    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.value.to_be_bytes()));
        bytes
    }

    // Deserialize from bytes
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 4 { return None; }
        let value = u32::from_be_bytes([
            data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?
        ]);
        Some(Self { value })
    }

    // Choose storage strategy
    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table  // For dense data
        // ComponentStorage::Sparse  // For marker components
    }
}
```

### Converting to Component

```rust
impl MyComponent {
    pub fn to_component(&self, env: &Env) -> Component {
        Component::new(Self::component_type(), self.serialize(env))
    }
}
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Rust version errors | `rustup update && rustup default stable` |
| WASM target missing | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI not found | `brew install stellar-cli` (macOS) |
| Dependency conflicts | `cargo update && cargo clean && cargo build` |
| Test snapshots issues | Delete `test_snapshots/` directory |

### Full Verification

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test && stellar contract build
```

---

## References

| Resource | Link |
|----------|------|
| Soroban Docs | [developers.stellar.org](https://developers.stellar.org/docs/build/smart-contracts) |
| Stellar CLI | [CLI Documentation](https://developers.stellar.org/docs/tools/cli) |
| Cougr Repository | [github.com/salazarsebas/Cougr](https://github.com/salazarsebas/Cougr) |
| Rust Testing | [Rust Book Ch. 11](https://doc.rust-lang.org/book/ch11-00-testing.html) |

---

## License

Part of the Cougr project. See main repository for license information.
