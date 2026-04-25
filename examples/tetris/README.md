# Tetris Smart Contract

An on-chain Tetris game implementation using the Cougr-Core ECS framework on Stellar's Soroban platform.

## 📋 Overview

This example demonstrates how to build a fully functional game as a smart contract using:
- **Soroban** - Stellar's smart contract platform
- **Cougr-Core** - ECS framework for on-chain games
- **Rust** - Smart contract programming language

## 🎮 Game Features

| Feature | Description |
|---------|-------------|
| **Game Board** | 20x10 grid with collision detection |
| **Tetrominoes** | All 7 classic shapes (I, J, L, O, S, T, Z) |
| **Rotation** | Full 360° rotation system |
| **Line Clearing** | Automatic detection and scoring |
| **Scoring** | Points based on lines cleared |
| **Leveling** | Difficulty increases every 10 lines |

## 🚀 Quick Start

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| Rust | 1.70.0+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| Stellar CLI | Latest | `cargo install --locked stellar-cli --features opt` |
| WASM Target | - | `rustup target add wasm32v1-none` |

### Build & Test
```bash
cd examples/tetris

# Build the contract
cargo build --release

# Run tests
cargo test

# Build for Soroban
stellar contract build
```

## 📦 Deployment

### Testnet Deployment
```bash
# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/tetris.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

**Deployed Contract:**
- **Network**: Stellar Testnet
- **Contract ID**: `CBWENGWFZHPNJPIHQAHXE5K34BGV2G5MOQIQ24PE44M6P42YULMQZYSF`
- **Explorer**: `https://stellar.expert/explorer/testnet/contract/CBWENGWFZHPNJPIHQAHXE5K34BGV2G5MOQIQ24PE44M6P42YULMQZYSF`

### Invoke Functions
```bash
# Initialize a new game
stellar contract invoke \
  --id CBWENGWFZHPNJPIHQAHXE5K34BGV2G5MOQIQ24PE44M6P42YULMQZYSF \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- init_game

# Move piece left
stellar contract invoke \
  --id CBWENGWFZHPNJPIHQAHXE5K34BGV2G5MOQIQ24PE44M6P42YULMQZYSF \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- move_left

# Update game tick (gravity + line clearing)
stellar contract invoke \
  --id CBWENGWFZHPNJPIHQAHXE5K34BGV2G5MOQIQ24PE44M6P42YULMQZYSF \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- update_tick
```

## 🏗️ Architecture

This example uses Cougr as the core architecture. `SimpleWorld` owns all mutable game state; systems are plain functions that operate on it; the world is persisted to Soroban instance storage alongside a lightweight `GameState` metadata struct.

### Components (`src/components.rs`)

| Component | Data | Used by |
|-----------|------|---------|
| `PieceComponent` | shape, x, y, rotation | Active tetromino entity |

### Systems (`src/systems.rs`)

| System | Responsibility |
|--------|---------------|
| `collision_system` | Returns true if a piece position/rotation overlaps the board or walls |
| `gravity_system` | Queries the active piece entity and moves it down one row |
| `lock_system` | Writes the piece onto the board, clears full lines, despawns the entity |

### Runtime shape

```
init_game()
  └─ SimpleWorld::new()
  └─ spawn_entity() → attach PieceComponent
  └─ save world + GameState to instance storage

update_tick() / move_left() / move_right() / rotate() / drop()
  └─ load world from storage
  └─ run system (gravity_system / shift_piece / lock_system)
  └─ save world + GameState back to storage
```

State that doesn't belong to any entity (score, level, next piece, game-over flag) lives in `GameState`, stored separately — the same pattern used by `flappy_bird` and `geometry_dash`.

## 🧪 Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific function
cargo test test_rotate
```

### Test Coverage

| Test | Description |
|------|-------------|
| `test_init_game` | Verifies game initialization |
| `test_move_left` | Tests left movement |
| `test_move_right` | Tests right movement |
| `test_move_down` | Tests downward movement |
| `test_rotate` | Tests piece rotation |
| `test_update_tick` | Tests game tick and line clearing |
| `test_game_over` | Tests end game detection |

## 📁 Project Structure
```
examples/tetris/
├── Cargo.toml          # Dependencies & build config
├── .gitignore          # Git ignore patterns
├── README.md           # This file
└── src/
    ├── lib.rs          # Contract entry points, GameState, storage helpers
    ├── components.rs   # PieceComponent (TetrominoShape, position, rotation)
    └── systems.rs      # collision_system, gravity_system, lock_system
```

## 🔧 Configuration

**Cargo.toml**
```toml
[dependencies]
soroban-sdk = "25.1.0"
cougr-core = "1.0.0"
```

## 📚 Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Documentation](https://developers.stellar.org/)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Book](https://doc.rust-lang.org/book/)

## 🤝 Contributing

This example is part of the Cougr framework. Contributions are welcome!

## 📄 License

Licensed under MIT OR Apache-2.0