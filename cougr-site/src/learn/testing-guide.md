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
- Estimating resource costs before you deploy

Check back soon - or [watch the repository](https://github.com/salazarsebas/Cougr).
