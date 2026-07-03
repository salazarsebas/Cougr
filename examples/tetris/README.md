# Tetris On-Chain Game

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.


## Purpose and pattern

## Overview


This example demonstrates a falling-block board simulation on Soroban with Cougr ECS concepts. It remains transitional while the arcade examples converge on the canonical `snake` `GameApp` architecture.


## Public contract API

## Game Features


| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | `none` | `GameState` | Initializes an empty board and current/next pieces. |
| `move_left` | `none` | `bool` | Attempts to move the active piece left. |
| `move_right` | `none` | `bool` | Attempts to move the active piece right. |
| `move_down` | `none` | `bool` | Soft-drops the active piece or locks it if blocked. |
| `rotate` | `none` | `bool` | Attempts clockwise rotation. |
| `drop` | `none` | `u32` | Hard-drops and locks the active piece; returns rows dropped. |
| `update_tick` | `none` | `GameState` | Runs one gravity tick. |
| `get_state` | `none` | `GameState` | Returns stored board and piece state. |


## Architecture overview

## Quick Start


```text
contract entrypoint
  ├─ reads game state from Soroban storage
  ├─ applies input or tick systems
  └─ writes updated state back to storage
```


Board and active piece state are contract types. `components.rs` documents the piece components used for Cougr-facing structure; `systems.rs` owns movement, collision, locking, scoring, and tick helpers.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | Per-contract game state where used by this example. | Keeps small arcade state close to the contract instance. |
| Persistent storage | Player- or world-scoped state where the example needs durable keyed state. | Keeps game progress available across invocations. |
| Temporary storage | Not used. | The examples favor deterministic recalculation over ephemeral caches. |

## Main gameplay flow

1. Call the initialization function to create the starting state.
2. Submit an input action such as movement, jump, flap, rotation, or shoot.
3. Call the tick/update function to run deterministic simulation logic.
4. Query public getters for score, position, active state, or terminal status.
5. Stop when the game-over/completed condition is reached, or reset/reinitialize where supported.

## Cougr APIs used

## Deployment

### Testnet Deployment
```bash
# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32v1-none/release/tetris.wasm \
  --source <YOUR_SECRET_KEY> \
  --network <NETWORK>
```

- **Network**: Stellar Testnet
- **Contract ID**: `<CONTRACT_ID>`
- **Explorer**: `https://stellar.expert/explorer/testnet/contract/<CONTRACT_ID>`

### Invoke Functions
```bash
# Initialize a new game
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network <NETWORK> \
  -- init_game

# Move piece left
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network <NETWORK> \
  -- move_left

# Update game tick (gravity + line clearing)
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network <NETWORK> \
  -- update_tick
```

## Benefits of Using Cougr-Core

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


- `ComponentTrait` and custom component modules document the ECS data boundary.
- `SimpleWorld`, `SimpleQueryBuilder`, `GameApp`, `ScheduleStage`, or `SystemConfig` are used where this transitional example has already adopted the maintained runtime shape.
- Auth, privacy, ZK, and standards APIs are intentionally not used; these arcade examples focus on deterministic game logic.

## Build and test commands



## Testing

```bash
cargo test

stellar contract build
```

## Known limitations


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

## Project Structure
```
examples/tetris/
├── Cargo.toml          # Dependencies & build config
├── .gitignore          # Git ignore patterns
├── README.md           # This file
└── src/
    └── lib.rs          # Smart contract implementation
```

## Configuration

**Cargo.toml**
```toml
[dependencies]
soroban-sdk = "25.1.0"
cougr-core = "1.1.0"
```

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Documentation](https://developers.stellar.org/)
- [Cougr Repository](https://github.com/salazarsebas/Cougr)
- [Rust Book](https://doc.rust-lang.org/book/)

## Contributing

This example is part of the Cougr framework. Contributions are welcome!

## License


- Transitional code may preserve older storage or scheduling patterns for compatibility reference.
- No authentication, matchmaking, real-time rendering, or production randomness is included.
- One contract instance generally represents one game or one keyed set of player games.
- For new work, prefer the canonical `snake` module split and `GameApp` tick wiring.
