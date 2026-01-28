# Bomberman On-Chain Game

This example demonstrates how to implement a Bomberman game as a smart contract on the Stellar blockchain using Soroban and cougr-core.

## Overview

This contract implements a simplified version of the classic Bomberman game where players can:
- Initialize a game with a grid
- Move players around the grid
- Place bombs that explode after a timer
- Handle collisions and scoring
- Check for game over conditions

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

### Testing

```bash
# Run unit tests
cargo test
```

## Game Logic

The game uses the following key structures:

- **Grid**: 2D array representing the maze with walls, destructible blocks, power-ups
- **Players**: Position, lives, bomb capacity
- **Bombs**: Position, timer, explosion power
- **Explosions**: Temporary areas that damage players and destroy blocks

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
     --wasm target/wasm32-unknown-unknown/release/bomberman.wasm \
     --source <your-secret-key> \
     --network testnet
   ```

3. Note the contract ID for future invocations (e.g., `CA3D5KRYM6CB7OWQ6TWYRR3Z4T7GNZLKERYNZGGA5SOAOPIFY6YQGAXE`)

### Testing the Contract

Invoke functions to test gameplay:

```bash
# Initialize game
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
  -- \
  init_game

# Move player (directions: 0=up, 1=right, 2=down, 3=left)
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
  -- \
  move_player \
  --player_id 1 \
  --direction 1

# Place bomb
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
  -- \
  place_bomb \
  --player_id 1

# Update game tick (advances timers, processes explosions)
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
  -- \
  update_tick

# Get player score
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
  -- \
  get_score \
  --player_id 1

# Check game status
stellar contract invoke \
  --id <contract-id> \
  --source <your-secret-key> \
  --network testnet \
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
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/bomberman.wasm --simulate
```

## Architecture

The contract uses Soroban's storage system for persistence and cougr-core for game-specific logic. The game state is stored efficiently to minimize transaction costs while maintaining real-time gameplay mechanics.