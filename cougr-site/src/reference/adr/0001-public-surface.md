# ADR 0001: Curated Public Surface

## Status

Accepted

## Context

Cougr exposes a broad API. Without curation, adopters can easily mistake public visibility for stable-contract inclusion.

## Decision

Cougr keeps a curated onboarding path at the crate root and explicitly separates:

- root-level ECS onboarding re-exports
- `standards` as a Stable namespace
- `zk::stable` as the stable privacy namespace
- `accounts` as a Beta namespace
- `zk::experimental` as the explicit non-contract privacy namespace

## Consequences

- docs can name the golden path without pretending the whole crate is frozen
- advanced but useful namespaces remain available
- public visibility alone is no longer the compatibility signal
