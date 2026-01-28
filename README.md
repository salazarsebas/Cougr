# Cougr Core

Cougr Core is a Soroban-compatible ECS (Entity Component System) framework for building on-chain games on the Stellar blockchain. It brings Bevy-inspired ergonomics to smart contracts while remaining no_std and WASM-friendly.

- Website: https://soroban.stellar.org/
- Docs: coming soon

## Highlights

| Area | What you get | Why it matters |
|---|---|---|
| ECS primitives | Entities, Components, Systems, Resources, Events, Queries | Model complex game logic simply and safely |
| Soroban-native | Works with soroban-sdk types, storage, and environment | Ship directly as Stellar contracts |
| no_std + WASM | Optimized for wasm32 targets | Small binaries and predictable execution |
| Storage efficiency | Dense/sparse layouts, reduced reads/writes | Lower fees and rent on ledger |
| Safety | Strong typing and borrowing discipline | Fewer runtime errors and better maintainability |

## How Cougr Core improves code writing

| Pain without Cougr | With Cougr Core |
|---|---|
| Scattered storage reads/writes increasing fees | Structured component storage with predictable access patterns |
| Ad-hoc state passing across functions | Resource and event systems to coordinate logic |
| Complex borrow lifetimes across subsystems | SystemParam abstractions that encapsulate access rules |
| Boilerplate-heavy entity management | Ergonomic spawn/despawn and query APIs |
| Hard-to-test game logic | Isolated systems with deterministic inputs for unit tests |

Developers focus on intent (what the system does) rather than plumbing (how to wire storage and access), which leads to clearer code and easier reviews.

## Example contracts in this repo

| Example | Path | CI | Build command |
|---|---|---|---|
| Bomberman (on-chain) | examples/bomberman | macOS workflow with stellar contract build | stellar contract build |
| Space Invaders | examples/space_invaders | macOS workflow with Homebrew install | stellar contract build |
| Flappy Bird | examples/flappy_bird | Ubuntu workflow | cargo build --target wasm32-unknown-unknown |
| Tic Tac Toe | examples/tic_tac_toe | Ubuntu workflow | stellar contract build |

## Testnet deployments

| Contract | Network | Contract ID | Explorer |
|---|---|---|---|
| Bomberman | Testnet | CCJMHQZFJJGUFP6TUXNEZ2NYVHWGU73GVF4CW45PRRGBDILUB2QJJ7QY | https://lab.stellar.org/r/testnet/contract/CCJMHQZFJJGUFP6TUXNEZ2NYVHWGU73GVF4CW45PRRGBDILUB2QJJ7QY |

Run the Deploy to Testnet workflow (workflow_dispatch) after adding the secret STELLAR_SECRET_KEY to your repository settings. The workflow uploads deployment logs and outputs the contract_id.

## Quick start

Add to your Cargo.toml:

```toml
[dependencies]
cougr-core = "0.0.1"
```

Create and use a world:

```rust
use cougr_core::prelude::*;

let mut world = World::new();
let entity = world.spawn_empty();
let position = Component::new(symbol_short!("position"), position_data);
world.add_component_to_entity(entity.id(), position);
let entities = world.query_entities(&[symbol_short!("position")]);
```

## Architecture

Core modules:

- entity: allocation and lifecycle management with generation tracking
- component: typed components, IDs, and registry
- world: container for entities, components, and systems
- system: execution model for game logic with SystemParam
- storage: table and sparse storage backends
- resource: global resources accessible to systems
- event: decoupled communication via events
- query: fast filtered iteration over entities

## Development

- Build: cargo build
- Test: cargo test
- Build for Soroban: cargo build --target wasm32-unknown-unknown --release

The crate is configured for small WASM binaries (LTO, single codegen unit, opt-level = "z").

## Compatibility

- Rust: 1.70+
- Edition: 2021
- Target: wasm32-unknown-unknown
- Soroban SDK: 23.0.2

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome. Open an issue or PR.

## Resources

- Soroban: https://soroban.stellar.org/
- Stellar: https://developers.stellar.org/
- Rust: https://www.rust-lang.org/learn
