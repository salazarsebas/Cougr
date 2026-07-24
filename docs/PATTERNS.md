# Cougr Developer Problem & Pattern Guide

This document organizes Cougr patterns by **developer intent / problem first**, linking directly to module standards, architectural patterns, and worked examples.

---

## 🎯 Developer Intent Index

| Developer Goal / Problem | Recommended Cougr Pattern | Reference Document / Module |
| :--- | :--- | :--- |
| **"I want to structure a gameplay loop"** | `GameApp` entrypoint & explicit Stage Layout | [Stage Layout](#stage-layout) |
| **"I want hidden information or commit-reveal mechanics"** | Merkle & Commit-Reveal boundary separation | [docs/PRIVACY_MODEL.md](PRIVACY_MODEL.md) |
| **"I want role-based access control or passwordless sign-in"** | Account Kernel boundary composition | [docs/ACCOUNT_KERNEL.md](ACCOUNT_KERNEL.md) |
| **"I want optimal component storage & low gas execution"** | Table storage for hot loops, Sparse for markers | [Storage Guidance](#storage-guidance) |
| **"I want standardized token/item interoperability"** | Cougr Standards Layer interfaces | [docs/STANDARDS_LAYER.md](STANDARDS_LAYER.md) |

---

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

`battleship` is the canonical reference for this pattern (see [docs/PRIVACY_MODEL.md](PRIVACY_MODEL.md)).

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

- ECS/gameplay core ([docs/ECS_CORE.md](ECS_CORE.md))
- account/auth flows ([docs/ACCOUNT_KERNEL.md](ACCOUNT_KERNEL.md))
- privacy/ZK ([docs/PRIVACY_MODEL.md](PRIVACY_MODEL.md))
- standards/operational controls ([docs/STANDARDS_LAYER.md](STANDARDS_LAYER.md))

Do not let auth or ZK concerns leak into every system by default. Compose them at the boundaries where they are needed.

## When Not To Use ECS

Do not force ECS into contracts that are:

- tiny and single-entity
- mostly configuration/state-machine driven
- dominated by one-off administrative flows

If the problem is closer to a fixed state machine than a world simulation, a direct contract model may be simpler and cheaper.
