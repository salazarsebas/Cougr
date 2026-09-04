<p align="center">
  <img src="public/Cougr.png" width="120" alt="Cougr logo" />
</p>

<h1 align="center">Cougr</h1>

<p align="center">
  <strong>The on-chain game engine for Stellar - ECS, privacy, and account abstraction in one crate</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/cougr-core"><img src="https://img.shields.io/crates/v/cougr-core.svg" alt="crates.io" /></a>
  <a href="https://github.com/salazarsebas/Cougr/actions/workflows/core.yml"><img src="https://github.com/salazarsebas/Cougr/actions/workflows/core.yml/badge.svg" alt="Core CI" /></a>
  <a href="https://stellar.org"><img src="https://img.shields.io/badge/Stellar-Soroban-blue?logo=stellar" alt="Stellar" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.88%2B-orange?logo=rust" alt="Rust" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="License" /></a>
</p>

Cougr is the framework for building on-chain games on Stellar. It gives you an
Entity Component System (ECS), built-in ZK proof tooling, session keys, and contract
standards - all compiled to a single `no_std` crate that runs natively on Soroban.

## Why Cougr on Stellar

| Capability | Cougr on Stellar |
|---|---|
| Language | Rust |
| ECS runtime | `SimpleWorld`, `ArchetypeWorld`, `GameApp` |
| ZK / hidden state | Groth16 + BLS12-381 + Merkle **on-chain** (X-Ray host fns) |
| Account abstraction | Session keys, passkeys, social recovery |
| Contract standards | Ownable, AccessControl, Pausable, guards, batch |
| State indexing | Structured Soroban events (`COUGR` topic prefix) |
| Deployment tooling | `stellar contract build/deploy` (native CLI) |
| Local dev chain | Stellar local network / Quickstart |
| Client SDK | Soroban CLI + community SDKs |

Stellar's strengths translate directly to better games:
- **3–5 second finality** at fractions of a cent per transaction
- **X-Ray host functions** run Groth16 and BLS12-381 verification on the host, not in WASM - ZK games are cheap
- **Passkey / WebAuthn** auth via `secp256r1` lets players use Face ID rather than seed phrases

## Installation

```toml
[dependencies]
cougr-core = "1.1.0"
soroban-sdk = "25.1.0"
```

Or scaffold a whole project with the CLI:

```bash
cargo install cougr-cli

cougr new my-game --template starter
cd my-game
cargo test
```

`cougr new` generates a contract crate following the canonical
`lib.rs` / `components.rs` / `systems.rs` layout with a passing `GameHarness`
test suite. Four templates are available - `starter`, `turn-based`,
`hidden-info`, and `session-auth` - each derived from a canonical example. See
[`cli/README.md`](cli/README.md).

## Quick start - your first on-chain game in 30 lines

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cougr-core = "1.1.0"
soroban-sdk = "25.1.0"
```

```rust
#![no_std]

use cougr_core::component::ComponentTrait;
use cougr_core::game::SorobanGame;
use cougr_core::{impl_component_observed, impl_soroban_game};
use soroban_sdk::{contract, contractimpl, contracttype, Env};

// 1. Define a component - one macro call, no boilerplate
#[contracttype]
#[derive(Clone, Debug)]
pub struct Position { pub x: i32, pub y: i32 }
impl_component_observed!(Position, "position", Table, { x: i32, y: i32 });
//  ↑ Emits a (COUGR, set, position) Soroban event on every change
//    so off-chain clients track state without polling.

// 2. Wire the contract to an ECS world - one macro call
#[contract]
pub struct MyGame;
impl_soroban_game!(MyGame, "world");

// 3. Write game logic in plain Rust
#[contractimpl]
impl MyGame {
    pub fn spawn(env: Env) -> u32 {
        let mut world = MyGame::load_world(&env);
        let player = world.spawn_entity();
        world.set_typed_observed(&env, player, &Position { x: 0, y: 0 });
        MyGame::save_world(&env, &world);
        player
    }

    pub fn move_right(env: Env, player_id: u32) {
        let mut world = MyGame::load_world(&env);
        let mut pos: Position = world.get_typed(&env, player_id).unwrap();
        pos.x += 1;
        world.set_typed_observed(&env, player_id, &pos);
        MyGame::save_world(&env, &world);
    }

    pub fn position(env: Env, player_id: u32) -> Option<Position> {
        MyGame::load_world(&env).get_typed(&env, player_id)
    }
}
```

```bash
# Build, test, deploy
cargo test
stellar contract build
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/my_game.wasm \
  --source alice --network testnet
```

## Components: two patterns for every case

**Simple components** - fixed-size primitives, fastest storage, optional event emission:

```rust
use cougr_core::{impl_component, impl_component_observed};
use soroban_sdk::contracttype;

// No events (internal state)
#[contracttype] #[derive(Clone, Debug)]
pub struct Velocity { pub dx: i32, pub dy: i32 }
impl_component!(Velocity, "velocity", Table, { dx: i32, dy: i32 });

// With events (indexer-friendly)
#[contracttype] #[derive(Clone, Debug)]
pub struct Health { pub current: u32, pub max: u32 }
impl_component_observed!(Health, "health", Table, { current: u32, max: u32 });
```

**Rich components** - Soroban XDR codec, supports `Address`, `Vec`, `String`, `Option`,
nested structs. Use when a component contains types not expressible as fixed-size bytes:

```rust
use cougr_core::impl_rich_component;
use soroban_sdk::{contracttype, Address, Vec};

// Address field - requires XDR / impl_rich_component!
#[contracttype] #[derive(Clone, Debug)]
pub struct Ownership { pub owner: Address, pub co_owners: Vec<Address> }
impl_rich_component!(Ownership, "ownership");

// Inventory with dynamic items
#[contracttype] #[derive(Clone, Debug)]
pub struct Inventory { pub items: Vec<u32>, pub capacity: u32 }
impl_rich_component!(Inventory, "inventory");
```

## ECS runtime

```rust
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::Env;

let env = Env::default();
let mut world = SimpleWorld::new(&env);

// Spawn entities
let knight = world.spawn_entity();  // → 1
let dragon = world.spawn_entity();  // → 2

// Set components (typed, with storage-tier routing)
world.set_typed(&env, knight, &Health { current: 100, max: 100 });
world.set_typed(&env, dragon, &Health { current: 500, max: 500 });

// Query entities that have a component
let env_ref = &env;
let combatants = world.get_entities_with_component(&Health::component_type(), env_ref);

// Remove and despawn
world.remove_typed::<Health>(knight);
world.despawn_entity(dragon);
```

`ArchetypeWorld` is available for larger entity counts and batch queries. Both backends
share the same `ComponentTrait` interface.

## GameApp - plugin-based composition

For complex games, `GameApp` provides stage-based scheduling and plugin registration:

```rust
use cougr_core::app::{named_system, GameApp, ScheduleStage};
use cougr_core::SystemConfig;
use soroban_sdk::Env;

let env = Env::default();
let mut app = GameApp::new(&env);

app.add_systems((
    named_system("physics", |world, env| {
        // runs every tick in Update
    })
    .in_stage(ScheduleStage::Update),

    named_system("scoring", |world, env| {
        // runs after physics in PostUpdate
    })
    .with_config(SystemConfig::new().in_stage(ScheduleStage::PostUpdate)),
));

app.run(&env).unwrap();
```

## Zero-knowledge and hidden state

Cougr bundles the full ZK toolchain for on-chain games - all verification runs on
Stellar's X-Ray host functions at host speed:

```rust
use cougr_core::privacy::stable::*;

// Groth16 proof verification (BN254)
let result = verify_groth16(&env, &proof, &vk, &public_inputs);

// Commit-reveal for hidden moves
let commitment = pedersen_commit(&env, &secret_value, &blinding_factor);
// ... reveal later and verify

// Merkle proof for fog-of-war or inventory ownership
let valid = verify_merkle_proof(&env, &root, &leaf, &proof_path);
```

Stable ZK primitives: Groth16, BLS12-381, SHA256 Merkle, Poseidon Merkle, Pedersen commitments.

## Account abstraction

```rust
use cougr_core::auth::*;

// Session keys - players approve a scoped key for a game session,
// then interact without signing every transaction
let session = SessionBuilder::new(&env, &player_address)
    .with_scope(&[contract_id])
    .with_expiry(env.ledger().timestamp() + 3600)
    .build();

// Social recovery - guardians can restore access
// Multi-device - per-device policies with revocation
// Passkey / WebAuthn - Face ID / Touch ID via secp256r1
```

## Contract standards (`ops`)

Drop-in security primitives, identical to OpenZeppelin on EVM:

```rust
use cougr_core::ops::*;

// Ownership
let ownable = Ownable::new(&env, symbol_short!("owner"));
ownable.transfer_ownership(&env, &new_owner);

// Role-based access
let acl = AccessControl::new(&env, symbol_short!("acl"));
acl.grant_role(&env, &MINTER_ROLE, &minter_address);

// Emergency stop
let pause = Pausable::new(&env, symbol_short!("pause"));
pause.pause(&env);
```

## Project status

| Area | Status |
|---|---|
| ECS runtime (`SimpleWorld`, `ArchetypeWorld`, `GameApp`) | Stable |
| Standards (`ops`) | Stable |
| Privacy primitives (`privacy::stable`, `zk::stable`) | Stable |
| Accounts and session keys (`auth`) | Beta |
| Advanced ZK (`privacy::experimental`) | Experimental |

## Repository layout

| Path | Purpose |
|---|---|
| `src/` | Core framework |
| `cli/` | `cougr-cli` - the `cougr` binary and its embedded project templates |
| `examples/spawn_and_move/` | Canonical starter - spawn + movement with ECS events |
| `examples/tic_tac_toe/` | Turn-based game with rich components and address ownership |
| `examples/*/` | 20+ standalone game contracts (asteroids, chess, battleship, ...) |
| `internal/` | Crates compiled into `cougr-core` via include: ZK circuit builders, session UX, and the test sandbox |
| `tests/` | Integration, edge-case, and stress coverage |
| `docs/` | Architecture, API contract, maturity model, threat model |
| `docs/strategy/` | Product, business, and repository strategy documents |
| `research/` | ZK and account abstraction design notes |
| `cougr-site/` | Source for the [documentation site](https://salazarsebas.github.io/Cougr/) (mdBook) and the example showcase gallery |
| `packages/tokens/` | Design tokens (color, type, spacing) shared by the docs site and the showcase |
| `tools/preview-gen/` | Generates SVG previews for the showcase from example scenario data |
| `skills/` | `skills.sh`-compatible AI assistant Skills for building games with `cougr-core` |
| `scripts/` | Repository hygiene and release tooling |

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```

## Compatibility

| Item | Value |
|---|---|
| Rust | 1.88+ (MSRV enforced in CI) |
| Edition | 2021 |
| License | MIT |
| Primary SDK | `soroban-sdk` 25.1.0 |
| Target | `wasm32v1-none` (Soroban WASM) |

## Documentation

- **[salazarsebas.github.io/Cougr](https://salazarsebas.github.io/Cougr/)** - the full documentation site, including the example showcase gallery
- [ARCHITECTURE.md](ARCHITECTURE.md) - module structure and design rationale
- [CHANGELOG.md](CHANGELOG.md) - release history
- [ROADMAP.md](ROADMAP.md) - phased roadmap and current status
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - community standards and enforcement
- [docs/ECS_CORE.md](docs/ECS_CORE.md) - ECS runtime model
- [docs/PRIVACY_MODEL.md](docs/PRIVACY_MODEL.md) - ZK proof tiers
- [docs/ACCOUNT_KERNEL.md](docs/ACCOUNT_KERNEL.md) - session keys and recovery
- [docs/PATTERNS.md](docs/PATTERNS.md) - recommended gameplay patterns
- [docs/ONCHAIN_OFFCHAIN_BOUNDARY.md](docs/ONCHAIN_OFFCHAIN_BOUNDARY.md) - deciding what belongs on-chain
- [examples/README.md](examples/README.md) - example catalog and usage guide
