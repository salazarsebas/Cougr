# Cougr-Core Examples

This directory contains practical examples demonstrating how to use the **cougr-core** ECS framework to build on-chain games on the Stellar blockchain via Soroban.

## Available Examples

### 🎮 Tetris - On-Chain Game

A fully functional Tetris game implementation as a Soroban smart contract.

**Location:** `examples/tetris/`

**Features:**
- Complete Tetris game logic (7 tetromino types, rotation, collision, scoring)
- Integration with cougr-core ECS framework
- Efficient on-chain state management
- Comprehensive test coverage
- Deployable to Stellar Testnet

**Quick Start:**
```bash
cd tetris
cargo test
stellar contract build
```

**[View Full Documentation →](./tetris/README.md)**

---

## Getting Started

All examples use cougr-core as a dependency:

```toml
[dependencies]
cougr-core = {tag = "v0.0.1", git = "https://github.com/salazarsebas/Cougr.git"}
```

### Prerequisites

1. **Rust** (1.70.0+)
   ```bash
   rustup update
   ```

2. **Stellar CLI**
   ```bash
   cargo install --locked stellar-cli
   ```

3. **WASM Target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Building Examples

Each example can be built independently:

```bash
cd <example-name>
cargo build
stellar contract build
```

### Testing Examples

Run tests for any example:

```bash
cd <example-name>
cargo test
```

## Contributing

Want to add a new example? Great! Consider these game types:

- Puzzle games (Sudoku, 2048, etc.)
- Turn-based strategy
- Card games
- Board games (Chess, Checkers, etc.)

Follow the structure of the Tetris example for consistency.

## Resources

- [Cougr-Core Documentation](../README.md)
- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar CLI Guide](https://developers.stellar.org/docs/tools/cli)
