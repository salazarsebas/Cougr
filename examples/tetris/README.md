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
- **Contract ID**: `<CONTRACT_ID>`
- **Explorer**: `https://stellar.expert/explorer/testnet/contract/<CONTRACT_ID>`

### Invoke Functions
```bash
# Initialize a new game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- init_game

# Move piece left
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- move_left

# Update game tick (gravity + line clearing)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- update_tick
```

## 🎯 Benefits of Using Cougr-Core

### Traditional Soroban vs. Cougr-Core

| Aspect | Traditional Soroban | With Cougr-Core ECS |
|--------|-------------------|-------------------|
| **Code Organization** | Monolithic contract logic | Modular components & systems |
| **State Management** | Manual storage handling | Automatic entity-component management |
| **Game Logic** | Tightly coupled functions | Reusable, composable systems |
| **Scalability** | Difficult to extend | Easy to add new features |
| **Code Reuse** | Limited | High - components are portable |
| **Testing** | Complex integration tests | Unit testable components |

### Cougr-Core Advantages

1. **Entity-Component-System Pattern**
   - Separates data (components) from logic (systems)
   - Makes code more maintainable and testable
   - Enables parallel processing of game logic

2. **Simplified State Management**
```rust
   // Traditional Soroban
   env.storage().instance().set(&DataKey::GameState, &state);
   
   // With Cougr-Core
   world.spawn_empty()
       .insert(Position { x: 5, y: 0 })
       .insert(Tetromino { shape: Shape::I });
```

3. **Reusable Components**
   - Components can be shared across different game types
   - Systems can be reused for similar game mechanics
   - Reduces development time for new games

4. **Better Code Organization**
   - Clear separation of concerns
   - Easier to understand and debug
   - Modular architecture

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
    └── lib.rs          # Smart contract implementation
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