# Standards Layer

## Purpose

The standards layer introduces reusable, storage-aware contract primitives in the style of OpenZeppelin building blocks, but shaped for Cougr's Soroban-oriented single-crate model.

These modules are meant to be composed into application contracts and account flows without depending on any example project.

## Included Standards

### `Ownable`

- single-owner access primitive
- explicit initialization
- direct transfer and renounce flows
- typed ownership transition events

### `Ownable2Step`

- staged ownership handoff
- pending-owner tracking in storage
- explicit acceptance requirement before ownership changes
- cancellation support for abandoned handoffs

### `AccessControl`

- role-based authorization keyed by `Symbol`
- per-role admin delegation
- explicit grant, revoke, and renounce semantics
- default admin role for bootstrapping new modules

### `Pausable`

- storage-backed emergency stop flaf
- explicit paused and unpaused transitions
- guard methods for mutating entrypoints

### `ExecutionGuard`

- storage-backed execution lock
- suited for reentrancy-like protection and mutation serialization
- can be used as explicit enter/exit calls or as a scoped closure wrapper

### `RecoveryGuard`

- blocks sensitive flows while a recovery window is active
- generic enough to compose with account recovery or application-defined incident response

### `BatchExecutor`

- reusable batch length validation
- single-path execution semantics for collections of operations
- explicit empty and oversize rejection

### `DelayedExecutionPolicy`

- storage-backed delayed operation queue
- deterministic operation IDs
- readiness and expiry checks
- cancellation and execution events

## Storage and Namespacing

Each standards module is instantiated with a `Symbol` identifier.

That identifier becomes part of the storage key, which allows a single contract to host multiple independent instances of the same standard without collisions.

## Authorization Model

These modules do not assume hidden caller semantics.

Where authorization matters:

- `Ownable` and `Ownable2Step` require an explicit caller address
- `AccessControl` checks the caller against the relevant admin role
- `Pausable`, `RecoveryGuard`, and similar state machines leave the surrounding authorization decision to the integring contract

This is intentional. Cougr keeps authorization visible at the integration boundary instead of burying it in generic helpers.

In particular, the standards mutators do not call `caller.require_auth()`. They compare `caller` against stored state and emit events, but the integrating contract is responsible for authenticating the caller before invoking them. This includes `Pausable::pause` and `Pausable::unpause`, where the `caller` argument is used only for the emitted event.

## Error Semantics

The standards layer uses `StandardsError` for consistent negative-path behavior across integrations.

Important failure modes include:

- unauthorized caller
- duplicate initialization
- missing or mismatched pending owner
- duplicate role grant or missing role during revoke
- paused versus not-paused guard failures
- execution lock contention
- recovery-active guard failure
- empty or oversize batches
- delayed operation not ready, expired, already executed, or missing

## Maturity

Status: Stable

The standards layer is part of Cougr's frozen `1.0` stable contract. Integrators should still supply their own caller-auth composition where required, but the module interfaces and documented failure semantics are now part of the defended public surface.
