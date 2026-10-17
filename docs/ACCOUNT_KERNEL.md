# Account Kernel

## Purpose

The goal is to make authorization explicit, modular, and replay-safe while keeping the `accounts` namespace outside Cougr's frozen `1.0` stable contract.

## Core Model

The account subsystem is now organized around:

- `AccountKernel`
  - the orchestrator that runs signer verification, policy checks, and replay protection
- signer interfaces
  - `AccountSigner`
  - base implementations: direct owner auth, session auth, secp256r1 passkey auth
- policy interfaces
  - generic `Policy<C>`
  - base implementations for intent expiry, session enforcement, active device checks, and guardian checks
- signed intent schema
  - `SignedIntent`, `SignerRef`, `IntentProof`
- structured auth results
  - `AuthResult`, `AuthMethod`

## Signed Intent Schema

`SignedIntent` binds:

- target account
- signer reference
- action payload
- nonce
- expiry
- deterministic `action_hash`
- proof material

### Deterministic Hash Layout

The deterministic hash is derived from the following bytes,
concatenated in order and then SHA-256-hashed:

1. domain tag: the ASCII bytes `cougr.intent.v1`
2. `account.to_xdr()`
 3. `env.current_contract_address().to_xdrX`
4. nonce (8 bytes, big-endian u64)
5. expiry (8 bytes, big-endian u64)
6. signer kind (4 bytes, big-endian u32)
7. signer session key id (32 bytes)
8. `signer.label.to_xdr()`
9. `action.system_name.to_xdr()`
10. action data (raw bytes)

Symbols are hashed by their XDR bytes, not by their in-env `Val`
payload, so the hash is a pure function of the intent content for every
valid `Symbol` length (including names longer than 9 characters). The
account and contract addresses are part of the hash, so a signature over
an intent is bound to a specific account and contract and cannot be
replayed against another one.

## Replay Protection

Cougr uses two replay domains:

- per-account nonce tracking for direct owner auth and passkey auth
- per-session nonce tracking for session intents

The replay implementation lives in:

- [src/accounts/replay.rs](../src/accounts/replay.rs)
- [src/accounts/storage.rs](../src/accounts/storage.rs)

## Session Model

Session state now includes:

- unique `key_id`
- scoped allowed actions
- operation budget
- expiration timestamp
- `next_nonce`

Session enforcement requires all of:

- session exists
- action is in scope
- session is not expired
- operation budget remains
- intent nonce matches `next_nonce`

On success the session consumes one operation and advances `next_nonce`.

## Signers

Current base signer implementations:

- direct owner signer
  - uses `require_auth`
- session signer
  - explicit non-fallback session path evaluated by the kernel
- secp256r1 passkey signer
  - verifies signatures against registered passkeys

## Policies

The policy layer is intentionally reusable across account features.

Current base policies:

- `IntentExpiryPolicy`
- `SessionPolicy`
- `ActiveDevicePolicy`
- `GuardianPolicy`

This is how device and recovery support now live under the same policy model instead of ad hoc checks.

## Auth Results

`AuthResult` returns structured information instead of only `Result<(), AccountError>`.

Current fields:

- method used
- nonce consumed
- session key id, when applicable
- remaining operations, when applicable

## Integration Note

The account kernel is now consumed through the curated `accounts` / `auth`
surface directly.

The previous `GameWorld` wrapper was removed so `1.0.0` does not freeze anextra orchestration layer. Authorization should be composed explicitly at the
application layer around `GameApp`, `SimpleWorld`, and the account primitives.
