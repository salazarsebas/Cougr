# Cougr ECS Core

## Purpose

This document defines the defended conceptual model for Cougr's ECS runtime.

It is the answer to "what are the actual core primitives?" and "which path is the one new users should learn first?"

## Core Model

The stable conceptual model is:

- `Entity`: an opaque runtime identity
- `Component`: typed or raw data attached to entities
- `Query`: a declarative selection over entities by component presence
- `System`: logic that reads or mutates the world
- `CommandQueue`: deferred structural mutations
- `GameApp`: app-level orchestration over world + scheduler + plugins
- `RuntimeWorld` / `RuntimeWorldMut`: the shared backend contract for Soroban-first worlds

For Soroban gameplay contracts, the recommended path is:

- `app`
- `SimpleWorld`
- `SimpleQuery`
- `SimpleScheduler`
- `GameApp`

`ArchetypeWorld` is the alternate backend for heavier query workloads.

The shared stable overlap between those backends lives in:

- `ecs::RuntimeWorld`
- `ecs::RuntimeWorldMut`

## Backend Roles

### `SimpleWorld`

Use when:

- entity counts are modest
- table-backed scans dominate
- operational simplicity matters more than archetype migration costs

Cost profile:

- cheap add/remove/update
- indexed table and all-storage component lookups
- predictable query path for common gameplay loops

### `ArchetypeWorld`

Use when:

- multi-component queries dominate
- entity composition is relatively stable
- migration cost is acceptable in exchange for tighter query scopes

Cost profile:

- more expensive structural changes
- more selective scans for multi-component queries

## Learnability Rule

A new user should be able to learn the main Cougr runtime from:

1. `README.md`
2. `GameApp`
3. `SimpleWorld`
4. `SimpleQueryBuilder`
5. one or two canonical examples

If a concept requires diving outside the Soroban-first runtime path to understand basic gameplay flow, that is a product bug.
