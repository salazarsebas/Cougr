# Cougr Patterns

## Purpose

This document captures the recommended architectural patterns for new Soroban game contracts built on Cougr.

The goal is to standardize how teams structure worlds, systems, stages, and storage choices instead of relying on ad-hoc example interpretation.

## Find a pattern by problem

Start here if you know what you're trying to build but not which Cougr module answers it. Each row links to the module-level doc for full detail, and to a concrete, current example that demonstrates it.

| I want... | Use | Example | Read more |
|---|---|---|---|
| **Fairness** (a roll, a draw, an outcome no one can predict or bias) | `circuits::FairDiceBuilder` - on-chain Groth16-verified randomness (Experimental) | [`dice_duel`](../examples/dice_duel) | [Hidden Information Guidance](#hidden-information-guidance) below, [PRIVACY_MODEL.md](./PRIVACY_MODEL.md) |
| **Hidden information** (cards, ship positions, sealed bids - state some players shouldn't see) | `privacy::stable` commit-reveal + Merkle primitives | [`battleship`](../examples/battleship) (canonical), [`rock_paper_scissors`](../examples/rock_paper_scissors), [`hidden_hand`](../examples/hidden_hand), [`blind_auction`](../examples/blind_auction) | [Hidden Information Guidance](#hidden-information-guidance) below, [PRIVACY_MODEL.md](./PRIVACY_MODEL.md) |
| **To gate an action behind a role** (admin-only, minter-only, etc.) | `AccessControl` - role-based authorization with per-role admin delegation | - | [STANDARDS_LAYER.md § AccessControl](./STANDARDS_LAYER.md#accesscontrol) |
| **Off-chain-friendly real-time movement** (clients track state without polling) | `impl_component_observed!` - emits a `(COUGR, set, <component>)` event on every change | [`spawn_and_move`](../examples/spawn_and_move) (start here), [`snake`](../examples/snake) | [System Design](#system-design) and [Query Guidance](#query-guidance) below, `docs/ECS_CORE.md` |
| **A passwordless sign-in** (Face ID / Touch ID instead of a seed phrase) | `secp256r1` passkey signer, composed through `AccountKernel` | [`guild_arena`](../examples/guild_arena) | [ACCOUNT_KERNEL.md § Signers](./ACCOUNT_KERNEL.md#signers) |
| **A session players approve once, not per-transaction** | Session signer + `SessionPolicy` (scope, expiry, operation budget) | [`session_arena`](../examples/session_arena) | [ACCOUNT_KERNEL.md § Session Model](./ACCOUNT_KERNEL.md#session-model) |
| **Account recovery if a device is lost** | `GuardianPolicy` + `ActiveDevicePolicy` | [`guild_arena`](../examples/guild_arena) | [ACCOUNT_KERNEL.md § Policies](./ACCOUNT_KERNEL.md#policies) |
| **An emergency stop / pause switch** | `Pausable` | - | [STANDARDS_LAYER.md § Pausable](./STANDARDS_LAYER.md#pausable) |
| **To serialize mutations / guard against reentrancy-like issues** | `ExecutionGuard` | - | [STANDARDS_LAYER.md § ExecutionGuard](./STANDARDS_LAYER.md#executionguard) |
| **Delayed or timelocked execution** | `DelayedExecutionPolicy` | - | [STANDARDS_LAYER.md § DelayedExecutionPolicy](./STANDARDS_LAYER.md#delayedexecutionpolicy) |
| **To batch several operations safely** | `BatchExecutor` | - | [STANDARDS_LAYER.md § BatchExecutor](./STANDARDS_LAYER.md#batchexecutor) |
| **To decide what belongs on-chain at all** (which state and rules justify their cost, and which should stay client-side) | The five-question boundary framework, applied per piece of state | [`battleship`](../examples/battleship), [`snake`](../examples/snake), [`blind_auction`](../examples/blind_auction) | [ONCHAIN_OFFCHAIN_BOUNDARY.md](./ONCHAIN_OFFCHAIN_BOUNDARY.md) |
| **To know whether I even need ECS** | Direct contract model for small/config-driven contracts | - | [When Not To Use ECS](#when-not-to-use-ecs) below |
| **To pick table vs. sparse storage** | Table for hot-loop state, sparse for infrequent markers | - | [Storage Guidance](#storage-guidance) below |
| **A thin, explicit contract entrypoint / gameplay loop** | `GameApp` + explicit stage placement | [`spawn_and_move`](../examples/spawn_and_move), [`snake`](../examples/snake) | [Default Entry Point](#default-entry-point) and [Stage Layout](#stage-layout) below |

Everything below this point is the module-level architectural guidance the table above links into - organized by Cougr's internal structure rather than by problem, for readers who already know which area they're working in.

## Default Entry Point

Use `GameApp` as the default runtime entrypoint.

Recommended shape:

1. build the app
2. register plugins and startup systems
3. register tick systems into explicit stages, preferably with `named_system(...)` / `named_context_system(...)`
4. run one schedule tick per contract invocation that advances gameplay

This keeps the "contract entrypoint" thin and the gameplay loop explicit.

## Stage Layout

Cougr's recommended schedule is:

- `Startup`: one-time entity/resource setup
- `PreUpdate`: input decoding, action validation, turn preparation
- `Update`: core gameplay state transitions
- `PostUpdate`: scoring, derived-state maintenance, indexing side effects
- `Cleanup`: despawns, expiry handling, transient marker removal

Do not use cross-stage `before` / `after` dependencies. Stage order is already the primary contract between phases.

## System Design

Prefer small systems with one responsibility:

- validation systems should reject or mark invalid intent
- update systems should apply game-state transitions
- cleanup systems should remove expired markers or entities

Use context-aware systems when you need deferred structural changes:

- queue spawns during iteration
- queue despawns after collision passes
- queue marker additions that should apply after the current scan

Use plain world/env systems when the system only needs direct mutation and no command buffering.

## Query Guidance

Prefer `SimpleQueryBuilder` for gameplay queries that need:

- multiple required components
- negative filters
- sparse-component inclusion
- "any-of" matching

Guidelines:

- default to table-only queries for tight loops
- opt into sparse inclusion only when marker/tag data must participate
- choose required components carefully so the scheduler can use the narrowest candidate set

## Hidden Information Guidance

For hidden-state or commit-reveal contracts:

- keep the contract entrypoints thin and verification-oriented
- use `privacy::stable` Merkle and commit-reveal primitives instead of example-local crypto formats
- treat proof verification as a boundary concern, not as something every gameplay system needs to understand
- keep public derived state separate from private commitments and Merkle roots

`battleship` is the canonical reference for this pattern.

## Storage Guidance

Use table storage for:

- frequently scanned gameplay state
- canonical state that participates in core loops
- components used by `Update` systems on most ticks

Use sparse storage for:

- infrequent markers
- administrative tags
- components mostly accessed by targeted lookups instead of broad scans

If a component becomes part of the hot loop, move it to table storage instead of compensating with more complex query logic.

## Recommended Separation

Keep modules separated by concern:

- ECS/gameplay core
- account/auth flows
- privacy/ZK
- standards/operational controls

Do not let auth or ZK concerns leak into every system by default. Compose them at the boundaries where they are needed.

## When Not To Use ECS

Do not force ECS into contracts that are:

- tiny and single-entity
- mostly configuration/state-machine driven
- dominated by one-off administrative flows

If the problem is closer to a fixed state machine than a world simulation, a direct contract model may be simpler and cheaper.
