# Cougr 1.0 API Freeze

## Purpose

This document is the final release-gate summary for the frozen `1.0` contract.

## Stable Contract

The `1.0` stable contract includes:

- root ECS re-exports documented in [API_CONTRACT.md](API_CONTRACT.md)
- `prelude`
- `runtime`
- `app`
- `ops`
- `standards`
- `privacy::stable`
- `zk::stable`

## Beta Namespaces

The following namespaces remain supported but intentionally outside the stable guarantee:

- `auth`
- `accounts`
- higher-level proof-submission orchestration tied to experimental verification

## Excluded From Compatibility Guarantees

The following remain outside the `1.0` compatibility guarantee:

- `privacy::experimental`
- `zk::experimental`
- hazmat crypto helpers
- transition or support-only helpers documented outside the stable contract

## Accounts Decision

`accounts` remains in the crate, but is not part of the frozen `1.0` stable contract.
`auth` is the clearer Beta-facing domain alias for application code and also remains outside the stable guarantee.

The decision is recorded in [adr/0002-accounts-beta.md](adr/0002-accounts-beta.md).

## Privacy Decision

The frozen privacy subset is exactly `zk::stable`, mirrored by `privacy::stable`.

The decision is recorded in [PRIVACY_MODEL.md](PRIVACY_MODEL.md) and [adr/0003-privacy-split.md](adr/0003-privacy-split.md).

## Standards Decision

`standards` is included in the stable `1.0` contract, mirrored by `ops`.

## Root Surface Decision

The root crate is intentionally narrower than the full module graph.
Documentation should prefer:

- `app` for gameplay runtime
- `auth`, `privacy`, and `ops` for product-level domain adoption
- `runtime` for advanced ECS composition
- `query` / `archetype_world` for cache and state helpers that are not part of the smallest onboarding surface

## Notes

Release preparation and migration framing now live in:

- [../CHANGELOG.md](../CHANGELOG.md)
- [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)
