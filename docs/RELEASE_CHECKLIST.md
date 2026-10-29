# Release Checklist

## Purpose

This checklist records the release hardening gate for Cougr's `1.0` line.

It is intentionally practical: release readiness depends on evidence in the repo
and a package that can be consumed from crates.io.

Published crate:

- https://crates.io/crates/cougr-core

## API Contract

- [x] [API_CONTRACT.md](API_CONTRACT.md) matches the actual curated public surface
- [x] [API_FREEZE_1_0.md](API_FREEZE_1_0.md) matches the current freeze decisions
- [x] [COMPATIBILITY_PROMISES.md](COMPATIBILITY_PROMISES.md) matches maturity and namespace intent
- [x] [PUBLIC_GAPS.md](PUBLIC_GAPS.md) reflects the real known gaps
- [x] [tests/public_api_surface.rs](../tests/public_api_surface.rs) passes and covers the curated namespaces

## Runtime Story

- [x] `app` remains the clearest onboarding path
- [x] `SimpleWorld` and `ArchetypeWorld` remain the defended Soroban-first backends
- [x] `RuntimeWorld` and `RuntimeWorldMut` still describe the stable shared overlap

## Domain Story

- [x] `auth` clearly mirrors Beta account flows
- [x] `privacy::stable` and `privacy::experimental` still reflect the intended maturity split
- [x] `ops` clearly mirrors stable operational standards

## Documentation

- [x] [README.md](../README.md) reflects the current onboarding path
- [x] [ECS_CORE.md](ECS_CORE.md) reflects the current backend story
- [x] [PERFORMANCE.md](PERFORMANCE.md) reflects the current benchmark suite
- [x] canonical example READMEs still match the product story
- [x] [CHANGELOG.md](../CHANGELOG.md) summarizes the release accurately

## Verification

- [x] `cargo fmt`
- [x] `cargo test`
- [x] `cargo bench --no-run`
- [x] example crates designated as canonical references still pass their local tests

## Security And Risk

- [x] [THREAT_MODEL.md](THREAT_MODEL.md) still matches the shipped trust boundaries
- [x] [UNSAFE_INVARIANTS.md](UNSAFE_INVARIANTS.md) still matches the crate's current safety posture
- [x] no Beta or Experimental surface is accidentally described as stable

## Ship Decision

The `1.0.0` release is acceptable because:

- the curated onboarding path is coherent
- the stable contract is narrower than the total public graph
- remaining gaps are explicit and acceptable

Current repo status:

- `cougr-core` `1.0.0` has been published
- release story is coherent enough for a defended `1.0.0`
- remaining gaps are compatibility and maturity gaps, not onboarding ambiguity
- the only unchecked gate in this file should be an actually unrun verification step
