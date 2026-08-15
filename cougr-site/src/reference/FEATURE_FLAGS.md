# Cougr Feature Flags

## Purpose

This document groups Cougr feature flags by maturity and intended usage.

## Current Flags

| Flag | Maturity | Intended use | Notes |
|---|---|---|---|
| `debug` | Support-only | Local diagnostics and introspection | Exposes runtime snapshots and metrics that are not part of the stable product contract |
| `hazmat-crypto` | Experimental | Advanced ZK and cryptographic integrations | Enables low-level host crypto helpers; do not treat as part of the stable privacy promise |
| `testutils` | Non-contract support surface | Tests and explicit test-utility consumers | Enables testing helpers such as `MockAccount` |

## Policy

- feature flags do not automatically promote a surface into the stable contract
- test-only or support-only flags remain outside compatibility guarantees
- new security-sensitive flags should default to Beta or Experimental until their contracts are written down

## Relationship To Public Surface

The maturity of the feature flag should be interpreted together with:

- [MATURITY_MODEL.md](MATURITY_MODEL.md)
- [API_CONTRACT.md](API_CONTRACT.md)
- [COMPATIBILITY_PROMISES.md](COMPATIBILITY_PROMISES.md)

In the product-level facade:

- `auth` mirrors the Beta `accounts` surface
- `privacy::stable` and `privacy::experimental` mirror the split inside `zk`
- `ops` mirrors the stable `standards` surface
