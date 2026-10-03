# account_kernel_counter

**Canonical** example proving that `cougr add account-kernel` produces a
module that is usable in a real Soroban game, not just importable.

The game is a simple per-owner counter.  Every increment must be authorized
through the `account_kernel` module that was scaffolded by
`cougr add account-kernel`.

---

## How this project was generated

```bash
# 1. Scaffold the project from the session-auth template.
cougr new account_kernel_counter --template session-auth
cd account_kernel_counter

# 2. Drop in the account-kernel piece (writes src/account_kernel.rs
#    and adds `pub mod account_kernel;` to lib.rs).
cougr add account-kernel

# 3. Build and test.
cargo test
stellar contract build
```

`src/account_kernel.rs` has one change from the file written by step 2: the
`testutils::Address as _` import is gated with `#[cfg(test)]` instead of
`#[cfg(any(test, feature = "testutils"))]`.  The `testutils` feature is not
declared by this crate, so the original attribute causes Clippy to emit an
`unexpected_cfgs` error under `-D warnings`.  Moving the import under
`#[cfg(test)]` is the correct fix; everything else in the file is verbatim
piece output.

---

## Purpose and pattern

`account_kernel_counter` is the end-to-end proof that the `account-kernel`
piece integrates cleanly into a real game contract.  It strips away all game
mechanics so you can focus on:

1. **Register** – owner calls `register_session` once to approve a scoped
   session key via `account_kernel::build_scope` and `SessionStorage::store`.
2. **Authorize** – each `increment` call delegates to
   `account_kernel::authorize_action`, which runs the full
   `AccountKernel → SessionPolicy` pipeline (expiry check, op-budget
   decrement, nonce advancement).
3. **Reject** – expired sessions, replayed nonces, and exhausted budgets all
   surface as panics from `authorize_action`, preventing the counter from
   advancing.

---

## Authorization flow

```
register_session(owner, key_id, max_ops, expires_at)
  └─ account_kernel::build_scope()          ← unmodified piece
     └─ SessionBuilder → SessionScope
  └─ SessionStorage::store()

increment(owner, key_id, nonce, intent_expires_at)
  └─ account_kernel::authorize_action()     ← unmodified piece
     └─ AccountKernel::authorize_session()
        ├─ SessionPolicy: expiry check
        ├─ SessionPolicy: action-name allow-list
        ├─ SessionPolicy: nonce advancement
        └─ SessionPolicy: op-budget decrement
  └─ counter += 1  (only reached on Ok)
```

---

## Public contract API

| Function | Parameters | Returns | Description |
|---|---|---|---|
| `register_session` | `owner`, `key_id`, `max_ops`, `expires_at` | `()` | Approve a scoped session for this owner (requires owner auth) |
| `increment` | `owner`, `key_id`, `nonce`, `intent_expires_at` | `u32` | Increment counter if the session authorizes the `tap` action |
| `count` | `owner` | `u32` | Current counter value for the owner |

---

## Cougr APIs used

- `account_kernel::build_scope` – piece helper that calls `SessionBuilder`
- `account_kernel::counter_action` – piece helper that constructs the `tap` `GameAction`
- `account_kernel::authorize_action` – piece helper that calls `AccountKernel::authorize_session`
- `AccountKernel` – separates signer verification, policy evaluation, and replay protection
- `SessionStorage` – load / store session keys on-chain
- `SessionBuilder` – declare allowed actions, max operations, and expiry

---

## Test coverage

| Test | Path exercised |
|---|---|
| `in_policy_action_increments_counter` | **Allow** – valid session, correct nonces, counter advances |
| `expired_session_blocks_increment` | **Expiry** – ledger timestamp past `expires_at`, increment panics |
| `replayed_nonce_blocks_increment` | **Replay** – same nonce submitted twice, second increment panics |
| `budget_exhaustion_blocks_further_increments` | Op-budget runs to zero, next increment panics |
| `count_returns_zero_before_any_increment` | Baseline – uninitialized counter reads as 0 |

The piece's own unit tests (`account_kernel::tests::*`) also run as part of
`cargo test` and cover `build_scope`, expiry, and nonce isolation at the
function level.

---

## Build and test

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```

---

## When to use which example

| Use `account_kernel_counter` when… | Use `session_arena` when… |
|---|---|
| Proving `cougr add account-kernel` works end-to-end | Learning `SessionManager` lifecycle |
| Understanding the raw `AccountKernel` / `authorize_action` path | Need `approve` / `renew` / `fallback` patterns |
| Writing tests that exercise the piece's own helpers | Need `MockSession` testutils |

---

## License

MIT
