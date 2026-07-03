# fog_explorer

**Canonical** ZK circuit example demonstrating `circuits::fog_of_war` for private exploration.

## Purpose and pattern

This example showcases a verifiable "fog of war" map exploration. Players move around a private map commitment, proving that they only reveal cells within their current line-of-sight visibility radius, without publishing the full map layout or their exact positions on the public blockchain.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `init_map` | `width: u32`, `height: u32`, `visibility_radius: u32` | `MapConfig` | Initializes the map configuration and builds the ZK circuit specs. |
| `register_explorer` | `player: Address`, `explored_root: BytesN<32>` | `ExplorerState` | Registers a player with their starting empty explored map commitment. |
| `explore` | `player: Address`, `map_root: BytesN<32>`, `prior_explored_root: BytesN<32>`, `next_explored_root: BytesN<32>`, `origin_x: i32`, `origin_y: i32`, `tile_x: i32`, `tile_y: i32`, `proof: Groth16Proof` | `bool` | Verifies a transition proof, updating the player's explored map root if valid. |
| `explorer_state` | `player: Address` | `ExplorerState` | Retrieves the current exploration state of the given player. |

## Architecture overview

```
                         ┌───────────────┐
                         │    Player     │
                         └──────┬────────┘
                                │ Moves & Generates ZK Proof
                     ┌──────────▼──────────┐
                     │     FogExplorer     │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads Spec
                     ┌──────────▼──────────┐
                     │ circuits::          │
                     │  fog_of_war         │
                     └─────────────────────┘
```

The player proves off-chain that the update from `prior_explored_root` to `next_explored_root` only uncovers the tile `(tile_x, tile_y)` within a visibility radius of their player coordinates. The contract verifies this transition against the map commitment.

## Storage model

Player exploration commitments and map layouts are stored in **Instance Storage** on-chain. Using instance storage guarantees fast state reads and writes during hot loops.

## Main gameplay flow

1. **Map Config**: Call `init_map` to configure map dimensions and visibility boundaries.
2. **Registration**: Explorer calls `register_explorer` to record their initial map state.
3. **Exploration**: Explorer calls `explore` with a Groth16 proof to update their visible tiles and progress.

## Cougr APIs used

- `circuits::fog_of_war`: Configures and handles verification of private line-of-sight map calculations.
- `zk::experimental::{FogOfWarSnapshot, FogOfWarTransition}`: Input structs for ZK state transition logic.
- `zk::Groth16Proof`: Cryptographic proof data container.

## Recommended testing approach

Use `GameHarness` and `test_fixtures` to mock ZK inputs and pipeline proof bytes. This allows testing successful proof verification paths without running full proving ceremonies in unit tests.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Grid coordinates are simplified.
- Map generation and parsing are managed off-chain.
