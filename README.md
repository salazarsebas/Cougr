# Cougr Core

**Cougr Core** is a Soroban-compatible ECS (Entity Component System) framework designed for building on-chain video games on the Stellar blockchain.

## Overview

This crate provides a complete ECS architecture adapted for `no_std` and WebAssembly (WASM) compatibility, built on top of the **soroban-sdk**. It follows an architecture inspired by Bevy ECS, providing a modular and efficient foundation for decentralized gaming experiences.

## Features

- ✅ **Complete ECS Architecture**: Entities, Components, Systems, World, Resources, Events, and Queries
- ✅ **Soroban-SDK Integration**: Native support for Stellar smart contracts
- ✅ **no_std Compatible**: Works in constrained environments
- ✅ **WASM Ready**: Optimized for WebAssembly execution
- ✅ **Type-Safe**: Leverages Rust's type system for safety
- ✅ **Efficient Storage**: Optimized component storage systems

## Architecture

### Core Modules

- **entity**: Entity management with unique IDs and generation tracking
- **component**: Component types and registry for attaching data to entities
- **world**: Central ECS world containing all entities, components, and systems
- **system**: System trait and implementations for game logic
- **storage**: Efficient component storage (Table and Sparse storage)
- **resource**: Global resources accessible to systems
- **event**: Event system for communication between systems
- **query**: Query system for filtering entities by components

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cougr-core = "0.0.1"
```

### Basic Usage

```rust
use cougr_core::prelude::*;

// Create a world
let mut world = World::new();

// Spawn an entity
let entity = world.spawn_empty();

// Add components
let position = Component::new(
    symbol_short!("position"),
    position_data
);
world.add_component_to_entity(entity.id(), position);

// Query entities
let entities = world.query_entities(&[symbol_short!("position")]);
```

## Module Documentation

### Entity Module (`entity.rs`)

Provides entity management functionality:
- `EntityId`: Unique identifier with generation tracking
- `Entity`: Entity container with component tracking
- `EntityManager`: Handles entity lifecycle (spawn, despawn, lookup)

### Component Module (`component.rs`)

Defines component types and management:
- `Component`: Base component type with Soroban serialization
- `ComponentId`: Unique component type identifier
- `ComponentRegistry`: Manages component type registration
- `ComponentTrait`: Trait for implementing custom components

### World Module (`world.rs`)

Central ECS container:
- `World`: Main ECS world containing all entities and components
- Methods for entity/component management
- Resource and event management
- Query execution

### System Module (`system.rs`)

System execution framework:
- `System` trait: Define game logic systems
- `SystemParam`: Parameter types for systems
- Pre-built systems: MovementSystem, CollisionSystem, HealthSystem

### Storage Module (`storage.rs`)

Component storage implementations:
- `Storage`: Base storage type
- `TableStorage`: Dense storage for common components
- `SparseStorage`: Sparse storage for rare components

### Resource Module (`resource.rs`)

Global state management:
- `Resource`: Global resources accessible to all systems
- `ResourceTrait`: Trait for implementing custom resources
- Example: `GameState` resource

### Event Module (`event.rs`)

Event system for inter-system communication:
- `Event`: Base event type
- `EventReader`: Read events in systems
- `EventWriter`: Send events from systems
- Pre-built events: `CollisionEvent`, `DamageEvent`

### Query Module (`query.rs`)

Entity filtering and querying:
- `Query`: Filter entities by components
- `QueryState`: Cached query results
- `QueryBuilder`: Fluent query construction
- `QueryFilter`: Custom filter trait

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Building for Soroban

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Profile Configuration

The crate is optimized for small WASM binary size:
- LTO enabled
- Single codegen unit
- Size optimization (`opt-level = "z"`)

## Compatibility

- **Rust Version**: 1.70.0+
- **Edition**: 2021
- **Target**: wasm32-unknown-unknown
- **Soroban SDK**: 23.0.2

## License

Licensed under MIT OR Apache-2.0

## Contributing

This is part of the Cougr framework for on-chain gaming on Stellar. Contributions are welcome!

## Resources

- [Soroban Documentation](https://soroban.stellar.org/)
- [Stellar Documentation](https://developers.stellar.org/)
- [Rust Documentation](https://www.rust-lang.org/learn)