# ADR 0005: Session UX

## Status

Accepted

## Context

Repeatedly signing transactions degrades the user experience for on-chain games, especially those requiring frequent interactions (like real-time or turn-based strategy). Players expect a seamless experience akin to Web2 gaming without constantly engaging with a wallet prompt.

## Decision

We introduce session keys with a fluent `SessionBuilder` API to authorize specific game actions over a time-bound window without requiring repeated user prompts. `authorize_with_fallback` will be used for graceful degradation, allowing operations to fall back to direct authorization when a session is expired or unavailable.

## Consequences

- Massively improved gameplay experience for end users.
- Integration becomes slightly more complex to handle session lifecycles.
- Wallets and client integrations must support and manage session key delegation.
