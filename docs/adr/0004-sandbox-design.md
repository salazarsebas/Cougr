# ADR 0004: Sandbox Design

## Status

Accepted

## Context

We need a secure testing and simulation environment for contracts to allow developers to validate logic (like ECS events and ZK proofs) locally without full node deployments. The sandbox should emulate the target environment accurately.

## Decision

We introduce a test sandbox utilizing `no_std` and `alloc` alongside the Soroban `testutils` feature. This sandbox provides core modules for testing games (such as `GameHarness`, `Scenario`, `WorldFixture`, `ReplayLog`, and `SnapshotAssert`).

## Consequences

- Developers can write fast local tests with a familiar testing API.
- We must maintain parity between sandbox behavior and on-chain execution.
- Relies on the `testutils` feature flag being managed correctly in the crate.
