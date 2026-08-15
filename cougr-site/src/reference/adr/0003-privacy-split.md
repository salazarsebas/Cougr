# ADR 0003: Stable Privacy Subset With Experimental Verification

## Status

Accepted

## Context

Cougr contains both defensible privacy primitives and faster-moving proof-verification helpers. Treating them as one maturity tier would overclaim guarantees.

## Decision

Cougr formally splits privacy into:

- `zk::stable` for commitments, commit-reveal, hidden-state codecs, and Merkle verification
- `zk::experimental` for advanced proof verification, circuits, channels, recursive layouts, and hazmat helpers

## Consequences

- privacy claims can stay narrow and defendable
- advanced ZK work can continue without blocking the stable subset
- compatibility promises can be scoped precisely by namespace
