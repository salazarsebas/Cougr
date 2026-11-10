# Release Status

## Purpose

This document is the high-signal answer to one question:

- now that Cougr `1.0.0` is published, what is guaranteed?

It is intentionally shorter than the full release checklist.

## Current Assessment

Cougr `1.0.0` is published as `cougr-core` on crates.io:

- https://crates.io/crates/cougr-core

The published crate has a defendable stable contract for its curated product
surface. That guarantee still depends on maintaining strict scope control
between the stable contract and the broader public repository.

That claim is intentionally narrow:

- `app` is the default runtime surface
- `SimpleWorld` and `ArchetypeWorld` are the defended Soroban-first backends
- `RuntimeWorld` and `RuntimeWorldMut` define the shared stable overlap
- `ops` is the preferred stable standards namespace
- `privacy::stable` is the preferred stable privacy namespace

## What Is Stable

- ECS onboarding/runtime contract
- `app`
- `ops` / `standards`
- `privacy::stable` / `zk::stable`
- the documented backend story around `SimpleWorld`, `ArchetypeWorld`, `RuntimeWorld`, and `RuntimeWorldMut`

## What Is Deliberately Not Frozen

- `auth` / `accounts`

These remain public and supported, but they are not the product-learning path and they are not the strongest compatibility promise in the repo.

## What Is Explicitly Experimental

- `privacy::experimental`
- `zk::experimental`
- hazmat cryptographic helpers

## Remaining Honest Gaps

- the crate still exposes more public surface than the stable contract actually guarantees
- migration to the curated path is documented, but not enforced
- auth remains Beta
- advanced proof tooling remains Experimental
- example quality and manifest consistency still require tighter release discipline

## Release Readout

From a release perspective, the repo has moved from "promising but fuzzy" to a
published `1.0.0` with an explicit stable contract.

The remaining risk is mostly ongoing scope control:

- keeping users on the curated path
- resisting the temptation to overstate Beta or Experimental surfaces
- preserving the freeze discipline already documented elsewhere
