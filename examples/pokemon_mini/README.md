# Pokemon Mini On-Chain Game

A Pokemon-style mini game implemented as a Soroban smart contract using the **cougr-core** ECS framework on the Stellar blockchain.

## Overview

This example demonstrates how to build deterministic on-chain game logic with:

| Feature | Description |
|---------|-------------|
| **Tile Map** | 8x8 grid with deterministic tile placement |
| **Movement** | Grid-based with collision detection |
| **Encounters** | Deterministic triggering on TallGrass tiles |
| **Battle** | Turn-based 1v1 combat system |

---

## Why cougr-core?

| Aspect | Benefit |
|--------|---------|
| **ComponentTrait** | Standardized serialization for on-chain storage |
| **Type Safety** | Unique `Symbol` identifiers per component |
| **Storage Optimization** | Table vs Sparse storage strategies |
| **Code Reusability** | Proven ECS patterns |

---

## Prerequisites

| Tool | Installation |
|------|--------------|
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| WASM Target | `rustup target add wasm32v1-none` |
| Stellar CLI | `brew install stellar-cli` (macOS) |

---

## Quick Start

### Build

```bash
cd examples/pokemon_mini

# Development build
cargo build

# Build WASM contract
stellar contract build
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

---

## Contract Functions

### Initialization

| Function | Description |
|----------|-------------|
| `init_player()` | Create player at spawn (1,1) with starter creature |

### Movement

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `move_player` | `direction: u32` | `u32` | Move player (0=blocked, 1=moved, 2=encounter) |
| `get_tile` | `x, y: i32` | `u32` | Get tile type at position |
| `get_map_size` | - | `(i32, i32)` | Returns (8, 8) |

**Direction Values:** 0=Up, 1=Down, 2=Left, 3=Right

### State Queries

| Function | Returns | Description |
|----------|---------|-------------|
| `get_player_state` | `(x, y, moves, in_battle, hp)` | Player position and status |
| `get_creature_stats` | `(species, level, hp, max_hp, atk, def)` | Creature stats |
| `get_battle_stats` | `(wins, losses, escapes)` | Battle history |

### Combat

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_battle_state` | - | `(in_battle, player_hp, enemy_hp, turn, result)` | Current battle |
| `battle_action` | `action: u32` | `u32` | Execute action (0=invalid, 1=continue, 2=ended) |

**Action Values:** 0=Attack, 1=Defend, 2=Run

**Result Values:** 0=None, 1=Win, 2=Lose, 3=Escaped

---

## Map Layout

```
8x8 Grid:
+--------+
|WWWWWWWW|  W = Wall
|WS..WTTW|  S = Spawn
|W...WTTW|  T = TallGrass
|W..T.TTW|  . = Grass
|W......W|  ~ = Water
|WTTT.~~W|
|WTTT.~~W|
|WWWWWWWW|
+--------+
```

---

## Game Mechanics

### Encounter System (Deterministic)

Encounters trigger on TallGrass when: `(x + y + move_count) % 5 == 0`

### Battle Damage Formula

```
damage = max(1, attacker_atk - defender_def)
```

- Player always acts first
- Defending adds +3 to defense for that turn
- Winning heals creature fully

---

## Deployed Contract (Testnet)

> **Contract ID:** `<CONTRACT_ID>`

| Network | Status | Explorer |
|---------|--------|----------|
| Stellar Testnet | ✅ Live | [View on Stellar Lab](https://stellar-explorer.acachete.xyz/contract/<CONTRACT_ID>) |

---

## Deployment

### Testnet

```bash
# Generate keypair
stellar keys generate --global alice --network testnet

# Fund account
# Visit: https://friendbot.stellar.org/?addr=<YOUR_ADDRESS>

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/pokemon_mini.wasm \
  --source alice \
  --network testnet
```

### Playing

```bash
# Use the deployed contract
CONTRACT_ID="<CONTRACT_ID>"

# Initialize player
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- init_player

# Get player state (returns: [x, y, moves, in_battle, hp])
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- get_player_state

# Move right (direction: 0=Up, 1=Down, 2=Left, 3=Right)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- move_player --direction 3

# Attack in battle (action: 0=Attack, 1=Defend, 2=Run)
stellar contract invoke --id $CONTRACT_ID --source alice --network testnet -- battle_action --action 0
```

---

## Project Structure

```
examples/pokemon_mini/
├── Cargo.toml          # Dependencies (cougr-core, soroban-sdk)
├── README.md           # This documentation
├── .gitignore          # Ignore rules
└── src/
    ├── lib.rs          # Contract entry points + tests
    ├── components.rs   # ECS components using ComponentTrait
    ├── systems.rs      # Game logic (map, movement, battle)
    └── simple_world.rs # Entity-component storage
```

---

## References

| Resource | Link |
|----------|------|
| Soroban Docs | [developers.stellar.org](https://developers.stellar.org/docs/build/smart-contracts) |
| Stellar CLI | [CLI Documentation](https://developers.stellar.org/docs/tools/cli) |
| Cougr Repository | [github.com/salazarsebas/Cougr](https://github.com/salazarsebas/Cougr) |


---

## License

Part of the Cougr project. See main repository for license information.
