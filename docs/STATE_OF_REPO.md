# State Of Repo

## Purpose

This document is a high-signal status snapshot of Cougr after the `1.0.0`
publication.

It is meant to answer:

- what is clearly done
- what is intentionally Beta
- what is still an honest remaining gap

## Stable Product Story

Published crate:

- https://crates.io/crates/cougr-core

The current defended product story is:

- `app` is the default gameplay runtime surface
- `SimpleWorld` and `ArchetypeWorld` are the defended Soroban-first backends
- `RuntimeWorld` and `RuntimeWorldMut` define their stable shared overlap
- `ops` / `standards` are stable operational standards
- `privacy::stable` / `zk::stable` are the frozen privacy contract

## Beta Story

The following remain intentionally non-frozen:

- `auth` / `accounts`

These are supported, but they are not the primary product-learning path.

## Experimental Story

The following remain explicitly outside the stable guarantee:

- `privacy::experimental`
- `zk::experimental`
- hazmat cryptographic helpers

## What Improved

- curated runtime onboarding through `app`
- explicit domain namespaces through `auth`, `privacy`, and `ops`
- stronger scheduler model
- stronger query model, backend parity, and benchmark story
- modern declarative system registration through `named_system(...)` and `add_systems(...)`
- canonical examples aligned with the curated path, including hidden-information patterns
- release-facing docs: changelog, migration guide, release checklist

## Remaining Honest Gaps

- the crate still exposes more total public surface than the stable contract actually guarantees
- auth remains Beta
- advanced proof tooling remains Experimental
- some root re-exports still preserve advanced capabilities that are intentionally not part of the smallest onboarding story

## Practical Reading Order

For a new user:

1. [../README.md](../README.md)
2. [API_CONTRACT.md](API_CONTRACT.md)
3. [ECS_CORE.md](ECS_CORE.md)
4. [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) if starting from older imports
5. [examples/README.md](../examples/README.md)

For release review:

1. [API_FREEZE_1_0.md](API_FREEZE_1_0.md)
2. [COMPATIBILITY_PROMISES.md](COMPATIBILITY_PROMISES.md)
3. [PUBLIC_GAPS.md](PUBLIC_GAPS.md)
4. [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)
