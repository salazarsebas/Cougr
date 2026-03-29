# Pac-Man On-Chain Game

A functional Pac-Man implementation for the Stellar blockchain, demonstrating advanced ECS (Entity Component System) decomposition using **Cougr-Core**.

## Overview

This example showcases architectural best practices for on-chain games, focusing on separating state and logic into modular, testable components and systems.

### ECS Architecture

The game state is decomposed into specialized components:
- **PacMan**: Player position, direction, and spawn data.
- **Ghost**: AI state, behavioral modes (Chase/Frightened), and entity identification.
- **Maze**: World grid data and pellet tracking.
- **GameStats**: Global resources like score, lives, and power-mode timers.
- **GameStatus**: High-level match lifecycle tracking (Game Over/Won).

Gameplay logic is encapsulated into named systems:
- **PlayerMovementSystem**: Handles Pac-Man traversal and boundary wrapping.
- **GhostMovementSystem**: Manages ghost AI and pathfinding.
- **CollisionSystem**: Resolves interactions between Pac-Man and ghosts using `cougr-core` events.
- **CollectibleSystem**: Processes pellet consumption and power-up activation.
- **GameProgressSystem**: Monitors win/loss conditions and global timers.

## Prerequisites

- **Rust**: 1.81.0 or newer
- **WASM Target**: `rustup target add wasm32v1-none`
- **Stellar CLI**: `cargo install stellar-cli`

## Building and Testing

### Build WASM
```bash
cargo build --target wasm32v1-none --release
```

### Run Tests
```bash
cargo test
```

## Usage

### Contract Functions

- `init_game()`: Scaffolds the maze and spawns entities.
- `change_direction(direction)`: Updates Pac-Man's intended path.
- `update_tick()`: Executes the ECS systems to advance the game state.
- `eat_pellet()`: Manually trigger consumption logic at the current position.
- `get_game_state()`: Returns a snapshot of all components.

## Deployment

To deploy to the Stellar Testnet:

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/pac_man.wasm \
  --source <your-account> \
  --network testnet
```

## Cougr Patterns Demonstrated

1. **Component Decomposition**: Avoiding a monolithic `GameState` by grouping data by responsibility.
2. **System Isolation**: Moving logic out of contract methods into pure functions that operate on component references.
3. **Event Integration**: Using `cougr_core::event` to track collisions for external observers.
4. **Resource Management**: Treating global metrics as ECS resources.

## Development

Ensure code quality before contributing:
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

*This project serves as a reference implementation for Cougr-Core.*