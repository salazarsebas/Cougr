# session_arena

**Canonical** Cougr example for the `SessionManager` gameplay lifecycle: approve once, play frictionlessly, renew on expiry, fallback when stale.

> **Related example:** For the full mobile-first flow with passkey auth, combos, and rounds, see [`tap_battle`](../tap_battle/README.md).

## Purpose and pattern

`session_arena` is the minimal reference implementation of cougr-core session UX. It strips away game mechanics so you can focus on:

1. **Approve** — owner signs once to create a scoped session key
2. **Play** — many `tap` calls without wallet prompts
3. **Renew** — extend session before expiry (owner re-approves)
4. **Fallback** — continue playing via direct owner auth when the session expires

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

- `SessionBuilder` — declare allowed actions, max operations, and expiry
- `SessionManager::approve` — create scoped session after owner auth
- `SessionManager::execute_action` — gasless gameplay via session key
- `SessionManager::status` — poll remaining ops and renewal hints
- `SessionManager::renew` — extend absolute `expires_at` timestamp
- `SessionManager::fallback_execute` — session-first with direct-auth fallback
- `SessionStorage` — load session keys by owner and key ID
- `MockSession` (testutils) — helper for unit tests

## Build and test

```bash
cargo test
stellar contract build
```

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
