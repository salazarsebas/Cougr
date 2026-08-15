# Architecture

High-level overview of how Cougr is organized. For usage, see [README.md](README.md).

## Layers

```
┌──────────────────────────────────────────────────────────────┐
│           game::SorobanGame  (contract integration)          │  Contract layer
├──────────────────────────────────────────────────────────────┤
│               app::GameApp                                   │  Default runtime surface
├───────────┬───────────────┬──────────────────────────────────┤
│  ECS      │  Accounts     │  Standards      │  ZK Proofs     │
├───────────┴───────────────┴─────────────────┴────────────────┤
│  soroban-sdk 25.1.0  (no_std, WASM)                          │
└──────────────────────────────────────────────────────────────┘
```

**game::SorobanGame** (`src/game.rs`) bridges the ECS and Soroban contract models.
The `SorobanGame` trait provides `load_world` and `save_world` as default methods,
eliminating repetitive storage-key boilerplate from contract entrypoints. Wire up
once with `impl_soroban_game!(MyContract, "key")`.

The companion helpers `SimpleWorld::load_from_instance` and `save_to_instance`
are the underlying primitives when you want finer control.

**GameApp** (`src/plugin/mod.rs`) is the default onboarding layer for complex
games. It owns a `SimpleWorld`, the scheduler, plugin registration, and runtime
resources in one place.

## ECS

Two storage backends, same `ComponentTrait` interface:

| Backend | File | Strategy | Best for |
|---|---|---|---|
| **SimpleWorld** | `src/simple_world/` | `Map<(EntityId, Symbol), Bytes>` with dual Table/Sparse indexes | General use, small entity counts |
| **ArchetypeWorld** | `src/archetype_world/` | Groups entities by component signature | Large entity counts, batch queries |

Both support typed access (`get_typed<T>`, `set_typed<T>`) and raw access (`get_component`, `add_component`).

Supporting systems:

- **Query cache** (`src/query/`) — version-tagged, invalidates on world mutation
- **Hooks** (`src/hooks.rs`) — callbacks on component add/remove
- **Observers** (`src/observers.rs`) — event-driven reactions
- **Commands** (`src/commands.rs`) — deferred mutations during system execution
- **Scheduler** (`src/scheduler/`) — stage-based, dependency-aware system ordering
- **Change tracker** (`src/change_tracker.rs`) — per-component dirty flags
- **Plugins** (`src/plugin/`) — modular game logic bundles
- **Incremental storage** (`src/incremental/`) — only persist dirty entities

### Component definition

Three macros cover every component case:

| Macro | When to use |
|---|---|
| `impl_component!` | Fixed-size primitives (`i32`, `u32`, `u64`, `u128`, `u8`, `bool`, `bytes32`) |
| `impl_component_observed!` | Same as above, plus structured Soroban events on every `set` |
| `impl_rich_component!` | Complex types via XDR codec: `Address`, `Vec`, `String`, `Option`, nested structs |

`impl_rich_component!` requires `#[contracttype]` on the struct. The XDR serialisation is handled entirely by the Soroban SDK — no manual `serialize`/`deserialize` implementation is needed.

Rich components are stored in Soroban instance storage (not the ECS `Map`) but share the same entity ID space.

## ZK Proofs (`src/zk/`)

All ZK operations use Stellar Protocol 25 (X-Ray) host functions — the heavy crypto runs on the host, not in WASM.

- **Groth16** (`groth16.rs`) — proof verification via BN254 pairing
- **BLS12-381** (`bls12_381.rs`) — G1 add/mul/MSM, pairing checks
- **Poseidon2** (`crypto.rs`) — ZK-friendly hashing, behind `hazmat-crypto` feature
- **Merkle trees** (`merkle/`) — SHA256 and Poseidon variants, sparse trees, on-chain proofs
- **Pedersen** (`commitment.rs`) — commitment scheme for hidden state
- **Game circuits** (`circuits.rs`, `traits.rs`) — `GameCircuit` trait + pre-built circuits (Movement, Combat, Inventory, TurnSequence) + `CustomCircuitBuilder`
- **ECS integration** (`components.rs`, `systems.rs`) — `CommitReveal`, `HiddenState`, `ProofSubmission` components with verification systems

## Accounts (`src/accounts/`)

Account abstraction layer with pluggable implementations:

```
CougrAccount (trait)
├── ClassicAccount      — standard Stellar keypair
└── ContractAccount     — smart contract wallet
     ├── SessionStorage — persistent session keys
     ├── RecoveryStorage — guardian-based recovery
     ├── DeviceStorage  — multi-device key management
     └── Secp256r1Storage — WebAuthn/Passkey keys
```

Key traits: `CougrAccount`, `SessionKeyProvider`, `RecoveryProvider`, `MultiDeviceProvider`.

`SessionBuilder` provides a fluent API for constructing scoped session keys. `authorize_with_fallback` handles graceful degradation from session keys to direct authorization. See [ADR 0005](../reference/adr/0005-session-ux.md).

## Standards (`src/standards/`)

Reusable contract standards for integrations that need explicit operational controls:

- `Ownable` and `Ownable2Step` for owner-managed authority
- `AccessControl` for role-based authorization with delegated admins
- `Pausable` for emergency stops
- `ExecutionGuard` for serialized critical sections
- `RecoveryGuard` for blocking sensitive paths during recovery windows
- `BatchExecutor` for bounded multi-operation flows
- `DelayedExecutionPolicy` for time-delayed operation queues

Each standard instance is keyed by a caller-supplied `Symbol`, which keeps storage deterministic and avoids collisions when a contract composes multiple modules.

## Competitive Layers (workspace subcrates)

Three layers ship inside the single `cougr-core` crate. Implementation lives in
`src/{circuits,session,test}/`; `internal/cougr-core-*` workspace members use
stubs for isolated `cargo check -p` runs.

| Public module | Source | Maturity | Feature |
|---|---|---|---|
| `cougr_core::circuits` | `src/circuits/` | Experimental | always |
| `cougr_core::session` | `src/session/` | Beta | always |
| `cougr_core::test` | `src/test/` | Beta | `testutils` |

Circuit builders: `hidden_cards`, `fog_of_war`, `fair_dice`, `sealed_bid` →
`GameCircuitSpec`. Examples: `hidden_hand`, `fog_explorer`, `dice_duel`,
`blind_auction`. See [ADR 0006](../reference/adr/0006-game-circuit-suite.md).

The test sandbox uses `no_std` + `alloc` with Soroban `testutils` — not `std`.
Enable with `cougr-core` feature `testutils`. Modules: `GameHarness`, `Scenario`,
`WorldFixture`, `ReplayLog`, `SnapshotAssert`. See [ADR 0004](../reference/adr/0004-sandbox-design.md) and [ADR 0007](../reference/adr/0007-workspace-subcrates.md).

## Feature Flags

| Flag | Enables |
|---|---|
| `hazmat-crypto` | Poseidon2 hash, BN254 curve ops (via `soroban-sdk/hazmat-crypto`) |
| `testutils` | `cougr_core::test` sandbox, `MockAccount`, Soroban test helpers |
| `debug` | Runtime introspection, metrics, state snapshots (`src/debug/`) |

## Build

Release builds are configured with LTO, `opt-level = "z"`, and `overflow-checks = true` to keep artifacts optimized for constrained execution environments.

Primary target: `wasm32v1-none`.
