# session_arena

**Canonical** example demonstrating `SessionManager` session lifecycles, scoped session keys, and fallback authorization.

## Purpose and pattern

This example showcases the onboarding pattern for friction-free session keys on Soroban. A player approves a session key once, enabling them to play without wallet confirmation prompts for subsequent transactions. If the session expires, the client can fall back to direct owner authentication.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `approve_session` | `owner: Address`, `max_taps: u32`, `expires_in: u64` | `ActiveSession` | Approves a new session key with specific action scopes and expiration constraints. |
| `tap` | `owner: Address`, `key_id: BytesN<32>` | `u32` | Increments the player's score, verified using the active session key (no wallet prompt). |
| `renew_session` | `owner: Address`, `key_id: BytesN<32>`, `expires_in: u64` | `ActiveSession` | Extends the active session key expiration window (requires owner wallet authorization). |
| `fallback_tap` | `owner: Address`, `key_id: BytesN<32>` | `u32` | Performs a tap action, falling back to direct owner wallet authorization if the session key is expired. |
| `score` | `owner: Address` | `u32` | Retrieves the current tap score of the player. |

## Architecture overview

```
                        ┌──────────────────┐
                        │   Player Owner   │
                        └────────┬─────────┘
                                 │ Approves
                     ┌───────────▼───────────┐
                     │    Active Session     │
                     │ (Temporary Key File)  │
                     └───────────┬───────────┘
                                 │ Authorizes (No Prompts)
                     ┌───────────▼───────────┐
                     │     SessionArena      │
                     │  (Soroban Contract)   │
                     └───────────┬───────────┘
                                 │ Updates
                     ┌───────────▼───────────┐
                     │         Score         │
                     │      (Component)      │
                     └───────────────────────┘
```

The game utilizes Cougr's session authentication module. The owner delegates authority for a specified duration and subset of actions to a local keypair. The contract verifies each transaction signature against the delegated session key metadata.

## Storage model

Session parameters and player score components are stored in **Instance Storage** on-chain. Session keys are designed to be temporary, so their lifecycle is optimized for minimum storage fee footprints.

## Main gameplay flow

1. **Authorization**: Owner calls `approve_session` from their wallet to authorize a temporary key for the `tap` action.
2. **Gameplay**: Client signs and executes calls to `tap` using the session key, bypassing any ledger signature popups.
3. **Renewal / Fallback**: If the session expires, the client calls `renew_session` (wallet-prompted) or calls `fallback_tap` (which handles direct-auth fallback).

## Cougr APIs used

- `SessionManager`: Manages approval, status, execution verification, and direct-auth fallbacks.
- `SessionBuilder`: Scopes maximum operations and timestamps for the generated key.
- `SessionStorage`: Loads active session state from the environment.
- `impl_component!`: Declares the player's `Score` component.

## Recommended testing approach

Use the `GameHarness` and `MockSession` to simulate wallet authorization, session approvals, key rotation, and operation expirations. Verification covers the happy-path tapping, time-bound key expiry, and fallback routing checks.

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- Simple score counter used for demonstrating the session auth wrapper; no actual gameplay engine included.
- Minimal session storage configuration.
