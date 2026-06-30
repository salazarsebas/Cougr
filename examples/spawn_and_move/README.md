# spawn_and_move

**Canonical** example demonstrating `SorobanGame`, `impl_component_observed!`, and typed ECS.

## Purpose and pattern

This example showcases a starter 2D grid world. A player can spawn an entity and move it in four directions. It demonstrates how to declare components, use observed components that automatically emit indexing events when modified, and load/save world state via `SorobanGame`.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `spawn` | - | `u32` | Spawns a new entity at origin `(0,0)` and returns its generated entity ID. |
| `move_entity` | `entity_id: u32`, `direction: u32` | - | Moves the entity in the specified direction if moves remain. |
| `position` | `entity_id: u32` | `Option<Position>` | Retrieves the current `Position` of the given entity. |
| `moves` | `entity_id: u32` | `Option<Moves>` | Retrieves the current `Moves` component of the given entity. |
| `entity_count` | - | `u32` | Retrieves the total number of entities spawned in the world. |

## Architecture overview

```
                    ┌────────────────────────┐
                    │  spawn_and_move Client │
                    └───────────┬────────────┘
                                │ Calls
                     ┌──────────▼──────────┐
                     │ SpawnAndMove Game   │
                     │ (Soroban Contract)  │
                     └──────────┬──────────┘
                                │ Loads / Saves
                     ┌──────────▼──────────┐
                     │     SimpleWorld     │
                     └──────────┬──────────┘
                                │ Stores
               ┌────────────────┴────────────────┐
        ┌──────▼──────┐                   ┌──────▼──────┐
        │  Position   │                   │    Moves    │
        │ (Observed)  │                   │  (Standard) │
        └─────────────┘                   └─────────────┘
```

When a client calls `move_entity`, the contract loads the `SimpleWorld` using `SpawnAndMove::load_world`, queries/modifies the `Position` and `Moves` components of the entity using typed ECS accessors, and saves the world state back using `SpawnAndMove::save_world`.

## Storage model

All game components and entity metadata are stored in Soroban **Instance Storage** via the underlying `SimpleWorld`. This keeps the hot-loop gameplay state localized and loaded efficiently in a single storage read/write lifecycle per transaction.

## Main gameplay flow

1. **Initialization / Spawn**: The user calls `spawn`. An entity is spawned at `(0,0)` with 10 moves remaining. A `("COUGR", "set", "position")` event is emitted.
2. **Action / Movement**: The user calls `move_entity` with `direction` (0=North, 1=East, 2=South, 3=West). The remaining moves decrement, the position updates, and a position set event is emitted.
3. **Query**: The user reads the entity's position or moves remaining.

## Cougr APIs used

- `SorobanGame` and `impl_soroban_game!`: Wires up standard boilerplate for loading/saving world state from instance storage.
- `impl_component_observed!`: Implements component layout with automated indexer events on set.
- `impl_component!`: Defines standard components without indexing event overhead.
- `SimpleWorld`: Provides structured, entity-component key-value management.

## Recommended testing approach

Tests in this project should utilize the `cougr-core` `testutils` feature, specifically `GameHarness` and `Scenario`. The `GameHarness` registers the contract, while the `Scenario` allows executing multi-step and multi-turn movement verification with intermediate assertions.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Simple grid coordinates without map size constraints or collision checks.
- Unauthenticated entity movement (any caller can move any entity ID).
