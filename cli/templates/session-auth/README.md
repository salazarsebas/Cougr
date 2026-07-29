# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, based on the canonical
[`{{source_example}}`](https://github.com/salazarsebas/Cougr/tree/main/examples/{{source_example}})
example.

> **Beta**: `cougr_core::session` is a Beta API. Expect the surface to move
> before 2.0.

## Purpose and pattern

Approve once, then play. The owner answers a single wallet prompt to create a
scoped session key; every move after that is authorized by the session instead
of the wallet. When a session expires mid-game the contract falls back to direct
owner auth rather than dropping the move. This is the pattern behind any game
that cannot ask for a signature on every turn.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `approve_session` | `owner: Address`, `max_taps: u32`, `expires_in: u64` | `ActiveSession` | The one wallet prompt: mint a scoped session key |
| `tap` | `owner: Address`, `key_id: BytesN<32>` | `u32` | Play a turn on the session; returns the new score |
| `fallback_tap` | `owner: Address`, `key_id: BytesN<32>` | `u32` | Play a turn, falling back to owner auth if the session is stale |
| `renew_session` | `owner: Address`, `key_id: BytesN<32>`, `expires_in: u64` | `ActiveSession` | Extend the session; same key ID |
| `session_state` | `owner: Address`, `key_id: BytesN<32>` | `Option<ActiveSession>` | Expiry, remaining budget, renewal hint |
| `score` | `owner: Address` | `u32` | Taps played so far |

## Architecture overview

```
lib.rs         contract entrypoints — auth, then gameplay state
  ├─ components.rs   Score component, player → entity storage key
  └─ systems.rs      session policy: allowed action, budget, lifetime, intent window
```

`systems.rs` is where a session's powers are defined — one allowed action, a
hard operation budget, an expiry, and a short intent window. Widening what a
session key can do is a visible edit to that file, not a change buried in an
entrypoint.

## Storage model

| Storage | Contents | Why |
| --- | --- | --- |
| Instance — session keys | Managed by `SessionManager` / `SessionStorage` | Scope, nonce, and expiry per key |
| Instance — `(player, Address)` | The player's ECS entity ID | Direct address → entity lookup, no scan |
| Instance — `world` | `SimpleWorld` with one `Score` per player | Gameplay state stays in the ECS |

## Main gameplay flow

1. The owner calls `approve_session(owner, 10, 3600)` and signs once. A session
   key is minted, scoped to the `tap` action with a budget of 10 operations.
2. The client stores the returned `key_id` and calls `tap` for each move. Each
   call builds a signed intent, the kernel checks scope, nonce, and expiry, and
   the tap counter increases — no wallet prompt.
3. When `session_state` reports `needs_renewal`, the client calls
   `renew_session`, which re-prompts the owner once and keeps the same key ID.
4. If a session lapses before renewal, `fallback_tap` authorizes the same move
   through direct owner auth so the turn is not lost.

## Cougr APIs used

| API | Why |
| --- | --- |
| `session::SessionManager` | Whole lifecycle — approve, execute, renew, fallback — in one facade |
| `accounts::SessionBuilder` | Declarative scope: allowed action, budget, expiry |
| `accounts::SignedIntent` / `ReplayProtection` | Nonce-protected intents for the fallback path |
| `session::ActiveSession` | Client-facing view of expiry and remaining budget |
| `impl_component!` | `Score` is one scalar per player |
| `SorobanGame` / `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `test::GameHarness`, `test::MockSession` | Session flows exercised in an integration test |

## Build and test

```bash
cargo test
stellar contract build
```

`stellar contract build` needs the WASM target and the Stellar CLI:

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
```

The build writes `target/wasm32v1-none/release/{{module_name}}.wasm`, which is
what you deploy with `stellar contract deploy`.

## Known limitations

* Sessions are scoped to a single action. A real game scopes several, and should
  keep the budget proportional to how long a session lives.
* There is no `revoke` entrypoint. `SessionManager::revoke` exists — wire it up
  before letting players sign in from shared devices.
* `fallback_tap` silently falls back. Production clients should surface that the
  session lapsed so the player knows to renew.
* Session keys here are held by the client. Pair with `Secp256r1Storage` and a
  passkey signer for a hardware-backed flow.
