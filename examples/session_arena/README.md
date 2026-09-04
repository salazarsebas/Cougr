# session_arena

 feat/session-manager-fresh
**Canonical** Cougr example for the `SessionManager` gameplay lifecycle: approve once, play frictionlessly, renew on expiry, fallback when stale.

> **Related example:** For the full mobile-first flow with passkey auth, combos, and rounds, see [`tap_battle`](../tap_battle/README.md).

## Purpose and pattern

`session_arena` is the minimal reference implementation of cougr-core session UX. It strips away game mechanics so you can focus on:

1. **Approve** - owner signs once to create a scoped session key
2. **Play** - many `tap` calls without wallet prompts
3. **Renew** - extend session before expiry (owner re-approves)
4. **Fallback** - continue playing via direct owner auth when the session expires

## Session lifecycle

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ approve_     │────▶│ tap (many    │────▶│ renew_       │────▶│ fallback_    │
│ session      │     │ times)       │     │ session      │     │ tap          │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
  owner auth           session key           owner auth          session or owner
  creates scope        no wallet prompt      extends expiry      auth fallback
```

> **Duration semantics:** `expires_in` and `expires_at` use **ledger timestamps** (seconds). `SessionBuilder::expires_in(n)` sets expiry to `ledger.timestamp() + n`.

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `approve_session` | `owner`, `max_taps`, `expires_in` | `ActiveSession` | One-time owner approval creating a scoped session |
| `tap` | `owner`, `key_id` | `u32` | Gameplay action via active session (no wallet prompt) |
| `renew_session` | `owner`, `key_id`, `expires_in` | `ActiveSession` | Extend session lifetime (owner must re-approve) |
| `fallback_tap` | `owner`, `key_id` | `u32` | Tap via session first, fall back to direct owner auth |
| `score` | `owner` | `u32` | Current tap count for the owner |

## Cougr APIs used

- `SessionBuilder` - declare allowed actions, max operations, and expiry
- `SessionManager::approve` - create scoped session after owner auth
- `SessionManager::execute_action` - gasless gameplay via session key
- `SessionManager::status` - poll remaining ops and renewal hints
- `SessionManager::renew` - extend absolute `expires_at` timestamp
- `SessionManager::fallback_execute` - session-first with direct-auth fallback
- `SessionStorage` - load session keys by owner and key ID
- `MockSession` (testutils) - helper for unit tests

## Build and test

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

 feat/session-manager-fresh
### Test coverage

| Test | Description |
|---|---|
| `approve_and_tap_without_reauth` | Multiple taps after single approval |
| `renew_session_extends_play_window` | Renew increases absolute `expires_at` |
| `fallback_tap_uses_direct_auth_after_session_expires` | Fallback after timestamp expiry |
| `mock_session_helper_matches_manager_flow` | `MockSession` matches manager flow |

## When to use which example

| Use `session_arena` when… | Use `tap_battle` when… |
|---|---|
| Learning SessionManager basics | Building a real game with passkeys |
| Prototyping session UX in a new game | Need combo mechanics and rounds |
| Writing integration tests for sessions | Demonstrating mobile-first auth flow |

---

## License

MIT

## Known limitations

- Simple score counter used for demonstrating the session auth wrapper; no actual gameplay engine included.
- Minimal session storage configuration.

