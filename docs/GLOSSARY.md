# Cougr Glossary

## Purpose

This document defines every Cougr-specific term precisely. Terms are used consistently across architecture docs, API contracts, examples, and the maturity model. If a term is not listed here, it is used in its ordinary English or Rust/Soroban sense.

---

## A

### AccountKernel
The orchestrator that runs signer verification, policy checks, and replay protection for the account subsystem. Lives in the `auth` / `accounts` namespace (Beta).

### Archetype (in ArchetypeWorld)
The signature (set of component types) that defines an entity group in `ArchetypeWorld`. Entities sharing the same archetype are stored contiguously for efficient batch queries.

### ArchetypeWorld
Alternate ECS world backend that groups entities by component signature (archetype). Best for large entity counts and multi-component queries where structural-change cost is acceptable. Shares the `ComponentTrait` interface with `SimpleWorld`.

### AuthResult
Structured authorization result returned by the account kernel. Includes the method used, nonce consumed, session key id, and remaining operations.

---

## B

### Beta
Maturity level for features that are usable and actively supported but expected to evolve outside the stable guarantee. Not SemVer-frozen. See [MATURITY_MODEL.md](MATURITY_MODEL.md).

---

## C

### Canonical example
A maintained reference architecture held to the full [EXAMPLE_STANDARD.md](../examples/EXAMPLE_STANDARD.md). Stays current as `cougr-core` evolves.

### Change tracker
Per-component dirty-flag system that records which entities had components modified since the last sweep. Used by incremental storage to persist only dirty entities.

### CommandQueue
Deferred structural-mutation buffer. Systems queue `spawn_entity`, `despawn_entity`, `add_component`, `remove_component` operations here instead of mutating the world directly during iteration.

### Commit-reveal
Pattern for hidden information. A player commits to a secret value (publishing a hash or Pedersen commitment), then later reveals the value and proves it matches the commitment. Supported by `privacy::stable`.

### Component
Typed or raw data attached to an ECS entity. Defined via one of three macros: `impl_component!` (fixed-size primitives), `impl_component_observed!` (same + Soroban events on every `set`), or `impl_rich_component!` (complex XDR types like `Address`, `Vec`, `String`). Per ECS convention, never called "entity data" or "property."

### Contract
A Soroban smart contract built on Cougr. The project standard is to say "contract" after first use, never "smart contract" in subsequent references.

### Contract layer
The `game::SorobanGame` integration layer that bridges the ECS world model and Soroban contract entrypoints.

### Curated surface
The defended set of public APIs that are part of Cougr's `1.0` stable contract. Documented in [API_CONTRACT.md](API_CONTRACT.md). Not every public Rust symbol is curated; only the documented stable namespaces are.

---

## D

### Default runtime surface
`app::GameApp` — the recommended entrypoint for new multi-system contracts. Provides stage-based scheduling, plugin registration, and runtime resource management.

---

## E

### Entity
An opaque runtime identity in the ECS. Entities have no data of their own; components are attached to them. Created via `world.spawn_entity()`, destroyed via `world.despawn_entity()`.

### Experimental
Maturity level for features that are exploratory, fast-moving, and not part of Cougr's stable promise. May change or be removed without compatibility guarantees. See [MATURITY_MODEL.md](MATURITY_MODEL.md).

---

## G

### GameApp
App-level orchestration that owns a `SimpleWorld`, scheduler, plugin registration, and runtime resources in one place. The default entrypoint for complex games. Lives in `cougr_core::app`.

---

## H

### Hidden state
Private game state encoded for selective disclosure. Players hold commitments or Merkle roots of hidden state and reveal only the subset needed for a given verification. Distinct from "secret" or "off-chain" state — hidden state is on-chain but unreadable without the reveal key.

### Hooks
Callbacks triggered on component `add` / `remove` events. A synchronous mechanism for enforcing invariants when components change.

---

## I

### impl_component!
Macro for fixed-size primitive components (e.g., `i32`, `u32`, `u64`, `bool`, `bytes32`). Stores directly in table storage.

### impl_component_observed!
Same as `impl_component!` but emits a structured `(COUGR, set, <name>)` Soroban event on every mutation. Used when off-chain indexers need to track state changes without polling.

### impl_rich_component!
Macro for complex types using Soroban's XDR codec. Supports `Address`, `Vec`, `String`, `Option`, and nested structs. Requires `#[contracttype]` on the struct.

### Incremental storage
Persistence layer that only flushes entities with dirty components (tracked by the change tracker). Reduces storage writes on large worlds where only a subset of entities change per tick.

---

## K

### Kernel. See AccountKernel.

---

## M

### Maturity model
The Stable / Beta / Experimental classification system governing compatibility promises, documentation depth, and test coverage across Cougr's public surfaces. Defined in [MATURITY_MODEL.md](MATURITY_MODEL.md). These three terms must be used identically everywhere — never synonyms ("production-ready" for Stable, "unstable" for Experimental, etc.).

### Merkle proof
An inclusion proof against a Merkle tree root. Used in hidden-information games (fog-of-war, private inventory) to prove state membership without full disclosure. Supported as a stable primitive in `privacy::stable`.

---

## O

### Observed component
A component defined with `impl_component_observed!` that emits structured Soroban events (topic prefix `COUGR`) on every `set` operation. The canonical choice when off-chain clients need to react to state changes.

### Observers
Event-driven reaction mechanism. Unlike hooks (synchronous, component-lifecycle), observers respond to arbitrary application-level events across system boundaries.

---

## P

### Plugin
A modular bundle of game logic (systems + setup) registered with `GameApp`. Enables composable feature toggles without editing core schedule code.

### Policy
Reusable authorization check in the account kernel. Base implementations: `IntentExpiryPolicy`, `SessionPolicy`, `ActiveDevicePolicy`, `GuardianPolicy`. Composed by the `AccountKernel` to enforce auth rules.

---

## Q

### Query
A declarative selection over entities by component presence. Cougr provides `SimpleQuery` / `SimpleQueryBuilder` for `SimpleWorld` and `ArchetypeQueryBuilder` for `ArchetypeWorld`. Both support `with_components`, `without_components`, and `with_any_components` filters.

### Query cache
Version-tagged result cache that invalidates on world mutation. Avoids re-scanning entities when the same query runs with no intervening writes.

---

## R

### Replay domain
Nonce tracking namespace. Cougr uses two replay domains: per-account nonce tracking (for direct owner auth and passkey auth) and per-session nonce tracking (for session intents).

### Rich component
A component using Soroban's XDR codec (via `impl_rich_component!`). Supports `Address`, `Vec`, `String`, `Option`, and nested structs. Stored in Soroban instance storage, not the ECS `Map`, but shares the same entity ID space.

### RuntimeWorld / RuntimeWorldMut
The shared trait contracts that both `SimpleWorld` and `ArchetypeWorld` implement. The backend-agnostic interface for Soroban-first worlds.

---

## S

### Scheduler
Stage-based, dependency-aware system ordering engine. `SimpleScheduler` runs systems in declared stage order (`Startup`, `PreUpdate`, `Update`, `PostUpdate`, `Cleanup`) with intra-stage `before`/`after` constraints.

### Session key
A scoped cryptographic key authorized for a limited set of actions, operation budget, and expiry window. Created via `SessionBuilder` (fluent API). Players interact without re-authorizing every action. The session UX pattern is Beta.

### SignedIntent
Structured authorization payload binding target account, signer reference, action payload, nonce, expiry, deterministic `action_hash`, and proof material. The canonical input to the `AccountKernel`.

### Signer
Base authorization implementation in the account subsystem. Variants: direct owner signer (`require_auth`), session signer (non-fallback session path), and secp256r1 passkey signer (WebAuthn/Passkey).

### SimpleWorld
Table-backed ECS world backend. Uses a `Map<(EntityId, Symbol), Bytes>` with dual table/sparse indexes. Best for general use with modest entity counts. The recommended default for new projects.

### SimpleQuery / SimpleQueryBuilder
Query builder API for `SimpleWorld`. Supports required components, negative filters, any-of matching, and sparse-component inclusion.

### SorobanGame
Trait providing `load_world` and `save_world` as default methods, eliminating repetitive storage-key boilerplate from contract entrypoints. Wired up once with `impl_soroban_game!`.

### Sparse storage
Storage tier for infrequent marker components, administrative tags, and data accessed via targeted lookups rather than broad scans. Complement to table storage.

### Stable
Maturity level for features that are SemVer-protected, documented with invariants and intended usage, covered by focused tests, and safe to present as part of Cougr's long-term public contract. See [MATURITY_MODEL.md](MATURITY_MODEL.md).

### Standards layer
Reusable contract primitives (Ownable, AccessControl, Pausable, ExecutionGuard, RecoveryGuard, BatchExecutor, DelayedExecutionPolicy) keyed by `Symbol` for deterministic storage. Stable. Also exposed as `ops`, the preferred domain alias.

### System
Logic that reads or mutates the world, registered into named stages. Prefer small systems with one responsibility (validation, state transition, cleanup). Per ECS convention, never called "script," "handler," or "processor."

---

## T

### Table storage
Storage tier for frequently scanned gameplay state and components used by `Update` systems on most ticks. The default storage for `impl_component!` and `impl_component_observed!`.

### Transitional example
An example written before the current standard or intentionally preserving an older pattern for compatibility reference. Still passes `cargo test` and `stellar contract build` but may not follow the latest module structure or README depth.

---

## X

### X-Ray host functions
Stellar Protocol 25 host functions for Groth16 (BN254) and BLS12-381 cryptographic operations. Run on the Soroban host, not in WASM. Enable cheap ZK proof verification for on-chain games.

---

## Z

### ZK proofs (zero-knowledge proofs)
Cryptographic proofs enabling hidden-state verification on-chain. Cougr bundles Groth16 (BN254 pairing), BLS12-381, Poseidon2 hashing, Merkle trees, Pedersen commitments, and prebuilt game circuits. Stable subset in `privacy::stable`; advanced verification in `privacy::experimental`.
