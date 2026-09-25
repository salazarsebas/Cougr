# cougr-sdk-session

A zero-dependency TypeScript session client that rebuilds the session authorization
[`cougr_core::accounts::SessionBuilder`](../../src/accounts/session_builder.rs) builds, so a web or
Node client can prepare a session, reject expired or replayed actions before submit, and agree with
the on-chain engine on every byte that matters.

The Rust engine remains the source of truth. This package does not re-interpret it: field meaning,
byte order, expiry units, and the replay nonce all match `src/accounts`.

## What is in here

| Path | Role |
|---|---|
| `src/session.ts` | `SessionBuilder`, `SessionClient`, `authorizeSession`, `deriveSessionKeyId`, `SessionError` |
| `src/index.ts` | Public entry point |
| `vectors/session-vectors.json` | Cross-language vectors, pinned by the Rust test and asserted here |
| `test/session.test.ts` | Builder, expiry boundary, budget, nonce, and scope tests |
| `test/vectors.test.ts` | Asserts the committed vectors against this implementation |

There is no build step. Node 22.6+ runs the TypeScript sources directly, so `npm test` needs no
toolchain and no install.

```bash
npm test          # node --test test/session.test.ts test/vectors.test.ts
```

## The model it mirrors

`SessionScope` and `SessionKey` keep the Rust shapes:

```text
SessionScope { allowedActions: string[], maxOperations: u32, expiresAt: u64 }
SessionKey   { keyId: BytesN<32>, scope, createdAt: u64, operationsUsed: u32, nextNonce: u64 }
```

Expiry is a ledger timestamp in **seconds**, and the boundary is the contract's: a session is expired
when `now >= expiresAt`. Replay protection is the per-session `nextNonce` counter that
`SessionStorage::consume_authorized_session` advances. `authorizeSession` runs the checks in the
contract's order — expiry, budget, nonce, allowed action — and returns the same numeric
`AccountError` codes (`SessionExpired = 21`, `NonceMismatch = 37`, `ActionNotAllowed = 38`,
`SessionBudgetExceeded = 39`, `SessionRevoked = 43`).

```ts
import { SessionBuilder, SessionClient } from 'cougr-sdk-session';

const clock = { timestamp: () => 1_735_689_700n, sequence: () => 42 };

const scope = new SessionBuilder(clock)
  .allowAction('move')
  .allowAction('attack')
  .maxOperations(100)
  .expiresIn(3_600n)
  .buildScope();
```

## The signer interface

`packages/sdk-core` has not landed, so this package does not fork the RPC stack. Anything that needs
a chain call takes a function argument instead:

```ts
import type { SessionKeyProvider, SessionScope } from 'cougr-sdk-session';

const provider: SessionKeyProvider = {
  createSession: (scope: SessionScope) => myRpcClient.createSession(scope),
};

const key = await new SessionBuilder(clock).allowAction('move').maxOperations(10).create(provider);
```

A provider only has to return a `SessionKey` shaped like the Rust struct, so any client — generated
bindings, a REST wrapper, or a test double — can implement it. `SessionClient` then guards the local
`nextNonce` and refuses a reused or future nonce with `NonceMismatch` before a transaction is built.

## Cross-language vectors

`vectors/session-vectors.json` is the shared artifact:

- `tests/session_vectors.rs` in the Rust crate derives the key id from
  `cougr_core::accounts::derive_session_key_id` and evaluates the real `SessionPolicy`, then asserts
  the committed vector is current.
- `test/vectors.test.ts` derives the same 32-byte id and runs the same accept/reject cases from the
  JSON.

The id layout, all big-endian, is `timestamp (u64) | sequence (u32) | existingSessions (u32) |
allowedActions.length (u32) | maxOperations (u32) | expiresAt (u64)`.

## Rust fields intentionally not wrapped

The package models the session surface, not the whole account kernel. These Rust fields and types
are deliberately absent:

| Rust item | Why it is not wrapped |
|---|---|
| `SessionKey.key_id` as an Ed25519 keypair | Key generation and custody belong to the wallet or signer; the client only carries the 32-byte id the contract knows. |
| `SignedIntent.action_hash` and `intent::{SignerRef, IntentProof}` | Hashing and signing are signer concerns. The package must not invent a second intent encoding that the contract would reject. |
| `AccountKernel::authorize_*` | This is a session client, not an account kernel; direct and passkey authorization are out of scope for this issue. |
| `SessionStorage::renew_session` / `revoke_session` | They are contract calls, not client-side rules. They go through the provider interface when `sdk-core` lands. |
| `max_operations` overflow behaviour | Rust `u32` arithmetic is checked by the contract; the client only validates the input range (`0..=u32::MAX`). |
| `symbol_short!` 9-character limit | The contract accepts any `Symbol` up to 32 characters; the client validates `^[a-z0-9_]{1,32}$` so `symbol_short!` actions work while leaving room for longer symbols. |

## Known limitations

- `deriveSessionKeyId` needs the ledger `timestamp`, `sequence`, and the number of existing sessions
  on that account; those are inputs because the package has no RPC dependency.
- `SessionClient` tracks one key in memory. Persisting it across page reloads is the caller's job.
- The contract remains the final authority: a client that passes every check here can still be
  rejected if it raced another submitter, which is why the error codes match.
