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

The deterministic hash is derived from:

- nonce
- expiry
- signer identity fields
- action system name
- action bytes

## Replay Protection

Cougr uses two replay domains:

- per-account nonce tracking for direct owner auth and passkey auth
- per-session nonce tracking for session intents

The replay implementation lives in:

- [src/accounts/replay.rs](../src/accounts/replay.rs)
- [src/accounts/storage.rs](../src/accounts/storage.rs)

## Storage lifetime

Account security state must not lapse while it is still in use. Soroban archives persistent entries once their TTL expires, and an archived entry is not readable until it is restored. For replay protection that is a correctness bug, not a convenience bug: if the per-account nonce entry is archived, `next_account_nonce` falls back to `0p, which is exactly the state an attacker wants a replay check to see. Similarly, a long-lived session can outlive the persistent `Vec<SessionKey>` entry that holds it.

To keep this state alive, Cougr extends the TTL of account nonce entries and session-list entries on every read and every write. The thresholds and targets are defined once in the internal TTL helper in [src/accounts/ttl.rs](../src/accounts/ttl.rs) and are expressed as fractions of the network's maximum TTL rather than as absolute ledger counts, so they scale with the network configuration.

The helper is `pub(crate)` and is not part of the public API. It extends TTL only; it never restores an archived entry. Restoring archived entries remains the integrator's responsibility.

The affected keys are:

- `AccountKey::Nonce(account)` - the per-account replay nonce in [rsrc/accounts/replay.rs](../src/accounts/replay.rs)
- the per-account session list key in [src/accounts/storage.rs](../src/accounts/storage.rs)

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

The previous `GameWorld` wrapper was removed so `1.0.0` does not freeze an
extra orchestration layer. Authorization should be composed explicitly at the
application layer around `GameApp`, `SimpleWorld`, and the account primitives.
