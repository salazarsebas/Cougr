# ADR 0002: Keep Accounts Out Of The Stable 1.0 Contract

## Status

Accepted

## Context

The account kernel, typed intents, replay domains, passkey support, and session enforcement are implemented. However, account abstraction remains a security-sensitive area with meaningful design-space risk.

## Decision

Cougr will keep `accounts` as a Beta namespace at `1.0` even though the kernel exists and is tested.

## Consequences

- the repo can truthfully document real implementation value
- maintainers keep room to tighten signer, policy, and integration contracts
- adopters are warned not to treat the current account API as SemVer-frozen
