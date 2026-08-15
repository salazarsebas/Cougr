# Cougr Compatibility Promises

## Purpose

This document defines the compatibility story Cougr is prepared to defend at `1.0`.

It turns the maturity model into explicit expectations for adopters, contributors, and maintainers.

## `1.0` Baseline

Cougr `1.0.0` freezes a scoped stable surface inside a broader public crate.

That means:

- compatibility promises are scoped by maturity, not by visibility alone
- stable, beta, and experimental namespaces can coexist in the same crate
- the stable guarantee is the documented contract, not every public symbol

## Stable Surfaces

The following surfaces are treated as Cougr's strongest `1.0` compatibility commitments:

- root ECS onboarding and runtime entrypoints documented in [API_CONTRACT.md](API_CONTRACT.md)
- `prelude`
- `runtime`
- `ops`
- `standards`
- `privacy::stable`
- `zk::stable`
- the contracts documented in [PRIVACY_MODEL.md](PRIVACY_MODEL.md) for commit-reveal, hidden-state codecs, and Merkle verification

For these surfaces, maintainers should preserve:

- type and function intent unless there is a documented breaking reason
- documented failure behavior
- documented malformed-input behavior where applicable
- byte-level or proof-shape contracts already written in the privacy model

## Beta Surfaces

The following surfaces are supported but intentionally not frozen:

- higher-level ECS helpers outside the frozen root/runtime contract
- `auth`
- `accounts`
- proof-submission orchestration that depends on experimental verification flows

For Beta surfaces, maintainers commit to:

- keep the product direction coherent
- document meaningful semantic changes
- avoid gratuitous churn
- preserve the curated onboarding path where practical

For Beta surfaces, maintainers do not yet promise:

- SemVer-stable signatures
- unchanged storage layouts for every helper
- unchanged auth or orchestration semantics across all releases

## Experimental Surfaces

The following surfaces are explicitly outside compatibility guarantees:

- `privacy::experimental`
- `zk::experimental`
- hazmat cryptographic helpers
- advanced proof-verification helpers and descriptors
- any public support surface documented as test-only or transition-only

These may:

- change shape
- move namespace
- be removed
- gain stronger validation that changes edge-case behavior

## Non-Contract Support Surfaces

Support-only surfaces such as `MockAccount` are not part of the default product contract.

They exist for tests and explicit utility consumers, not as long-term framework guarantees.

## Change Management Rules

When changing a Stable or Beta public surface, update at minimum:

- [MATURITY_MODEL.md](MATURITY_MODEL.md) if the classification changes
- [API_CONTRACT.md](API_CONTRACT.md) if the recommended contract changes
- [PUBLIC_GAPS.md](PUBLIC_GAPS.md) if a known gap is closed or newly introduced
- [THREAT_MODEL.md](THREAT_MODEL.md) if trust assumptions or security posture change

## `1.0` Freeze Decisions

The `1.0` release gate decisions are:

- ECS onboarding and runtime surfaces are in the stable contract
- `ops` is the stable domain alias for standards
- `standards` is in the stable contract
- `auth` is a Beta domain alias and is not part of the stable guarantee
- `accounts` remains Beta and is not part of the stable guarantee
- `privacy::stable` maps to the frozen privacy contract
- `zk::stable` is the frozen privacy contract
- `privacy::experimental` remains outside compatibility guarantees
- `zk::experimental` remains outside compatibility guarantees
