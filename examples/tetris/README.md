# Tetris On-Chain with Cougr ECS

This example demonstrates a fully functional Tetris game implemented as a Soroban smart contract on the Stellar blockchain. It leverages the `cougr-core` ECS (Entity Component System) framework to manage game state, logic, and entities.

## Overview

The contract implements standard Tetris rules including:
- 7 Tetromino shapes (I, J, L, O, S, T, Z)
- Piece rotation (SRS-lite)
- Line clearing and scoring
- Level progression
- Game over detection

It serves as a reference implementation for building complex logic using `cougr-core`.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Soroban CLI](https://transformers.stellar.org/docs/getting-started/setup#install-the-soroban-cli)
- [Stellar CLI](https://github.com/stellar/stellar-cli)

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/salazarsebas/Cougr.git
   cd Cougr/examples/tetris
   ```

2. **Install Dependencies:**
   ```bash
   cargo build
   ```

## Build

To build the optimized WASM contract:

```bash
stellar contract build
```

This will output the `.wasm` file in `target/wasm32-unknown-unknown/release/`.

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

## Deployment (Testnet)

1. **Configure Identity:**
   ```bash
   stellar keys generate --global alice
   stellar keys address alice
   ```

2. **Deploy Contract:**
   ```bash
   stellar contract deploy \
     --wasm target/wasm32-unknown-unknown/release/tetris.wasm \
     --source alice \
     --network testnet
   ```
   *Save the returned Contract ID (e.g., `CA...`).*

3. **Interact:**

   *Initialize Game:*
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source alice \
     --network testnet \
     -- \
     init_game
   ```

   *Move Left:*
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source alice \
     --network testnet \
     -- \
     move_left
   ```

   *Get State:*
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     --source alice \
     --network testnet \
     -- \
     get_state
   ```

## Documentation

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Cougr Core](https://github.com/salazarsebas/Cougr)
