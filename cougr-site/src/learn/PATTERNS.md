# Cougr Patterns

## Purpose

This document captures the recommended architectural patterns for new Soroban game contracts built on Cougr.

The goal is to standardize how teams structure worlds, systems, stages, and storage choices instead of relying on ad-hoc example interpretation.

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
