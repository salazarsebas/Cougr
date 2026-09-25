# Testing Guide

> ⏳ **This page is being written.**
>
> `GameHarness`, `Scenario`, and `SnapshotAssert` exist in the codebase and are used across thousands of lines of tests, but have no standalone guide yet. Named explicitly in `docs/strategy/08-ux-strategy.md` Stage 5.
>
> **Tracked in:** [`salazarsebas/Cougr` issues](https://github.com/salazarsebas/Cougr/issues)

---

## What this guide will cover

- Setting up `GameHarness` for unit tests
- Writing `Scenario`-based integration tests
- Using `SnapshotAssert` to lock in expected world state
- How the Soroban test sandbox works (and how it differs from running `cargo test` for a normal library)

Check back soon - or [watch the repository](https://github.com/salazarsebas/Cougr).

---

## Reporting resource cost

A passing test says a game behaves; Soroban fees are multi-dimensional, so it does
not say a call fits a ledger. `GameHarness` can report the budget snapshot the host
already meters for the last top-level invocation:

```rust
let report = harness.resource_report();
println!("{}", report.to_csv());
```

`ResourceReport` carries the eight dimensions `Env::cost_estimate().resources()`
returns - instructions, memory, disk read entries and bytes, write entries and
bytes, in-memory entries read, and contract event bytes. That is the snapshot the
testutils host already keeps rather than a second estimator. The sandbox registers a
Rust contract instead of a Wasm one, so the report covers host-side work only: VM
instantiation, Wasm reads, and rent bumps on real entries are not part of it.

To make a scenario a gate, commit its numbers with a tolerance:

```rust
let budget = ResourceBudget::new(baseline, 10, 0);
harness.assert_resource_budget("init_match", &budget);
```

`tests/resource_budget.rs` pins one turn-based scenario against
`tests/turn_based_resource_baseline.csv`, so `cargo test --workspace --features
testutils` - the command CI runs - fails when a dimension regresses past the
documented tolerance. The `init_match` row is the whole match setup and the
`alternating_move` row is a single move; metering resets per top-level invocation,
so one move is the unit. Percentages absorb compiler and host-version noise on
instructions, memory, and bytes. The entry counts get no extra slack, because a new
write or a loop over entries is a structural regression rather than noise.
