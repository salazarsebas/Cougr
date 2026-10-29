# Cougr Unsafe Invariants

## Purpose

This document records Cougr's current unsafe posture for the published crate.

## Current State

The shipped `cougr-core` crate currently contains no internal `unsafe` code paths.

The previous repository carried lower-level ECS internals with explicit unsafe boundaries, but those paths are not part of the current published implementation and are no longer part of the maintained source tree.

As a result, the main invariant for the current crate is simple:

- keep the crate free of internal `unsafe` unless a demonstrated Soroban-specific need justifies introducing it
- prefer safe Soroban SDK types and explicit ownership over low-level memory tricks
- treat any future `unsafe` introduction as a design-level change that requires dedicated review, tests, and documentation

## Host-Crypto Boundary

Cougr still depends on Soroban host functionality for BN254 and related cryptographic operations exposed by `soroban-sdk`.

That is a trust boundary, but it is not an internal `unsafe` boundary inside Cougr itself.

The invariants Cougr must preserve around that boundary are:

- validate shape constraints before calling host cryptographic helpers
- keep stable and experimental maturity boundaries explicit in the public API
- document limitations precisely when host primitives do not imply stronger guarantees beyond the Soroban SDK contract

## Review Checklist

Before merging a change that introduces `unsafe` into the crate, confirm:

- the safe alternative was evaluated and rejected for a concrete reason
- the exact invariant is documented inline in the code
- tests exercise the boundary and likely misuse cases
- this document and release-facing docs are updated to reflect the new reality
