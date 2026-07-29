# ADR 0004: Include Standards In The Stable 1.0 Contract

## Status

Accepted

## Context

The standards layer is documented, integration-tested, and intentionally designed as a reusable framework surface rather than example glue.

## Decision

Cougr includes `standards` in the stable `1.0` contract.

## Consequences

- integrators can treat `standards` as part of the defended public framework surface
- future changes to these modules now carry stable-contract weight
- authorization composition remains explicit at the integration boundary rather than hidden inside the primitives
