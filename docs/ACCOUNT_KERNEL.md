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

The deterministic hash is derived from the SHA-256 digest of the following concatenated byte layout:

1. domain tag (`b"COUGR_ACCOUNT_INTENT_V1"`, 23 ASCII bytes)
2. target account address (`account.to_xdr(env)`, Soroban `ScVal::Address` XDR bytes)
3. current contract address (`env.current_contract_address().to_xdr(env)`, Soroban `ScVal::Address` XDR bytes)
4. nonce (`u64` big-endian, 8 bytes)
5. expiry (`expires_at`, `u64` big-endian, 8 bytes)
6. signer kind discriminant (`signer.kind as u32` big-endian, 4 bytes: `0` = Direct, `1` = Session, `2` = Passkey)
7. signer session key ID (`signer.session_key_id`, 32 raw bytes)
8. signer label (`signer.label.to_xdr(env)`, Soroban `ScVal::Symbol` XDR bytes)
9. action system name (`action.system_name.to_xdr(env)`, Soroban `ScVal::Symbol` XDR bytes)
10. action bytes (`action.data`, raw `Bytes`)

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

The previous `GameWorld` wrapper was removed so `1.0.0` does not freeze an
extra orchestration layer. Authorization should be composed explicitly at the
application layer around `GameApp`, `SimpleWorld`, and the account primitives.
