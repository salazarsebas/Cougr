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

The deterministic hash is derived as `SHA-256(preimage)`, where `preimage` concatenates the following fields in exact order:

| Order | Field | Encoding | Length |
|---|---|---|---|
| 1 | Domain tag | ASCII bytes `"cougr:account:intent:v1"` | 23 bytes |
| 2 | Target account (`intent.account`) | Soroban XDR `ScVal::Address` (`account.to_xdr(env)`) | variable (44 bytes for classic `G…` account, 40 bytes for `C…` contract) |
| 3 | Verifying contract (`env.current_contract_address()`) | Soroban XDR `ScVal::Address` (`contract.to_xdr(env)`) | variable (40 bytes for `C…` contract) |
| 4 | Nonce (`intent.nonce`) | `u64` big-endian | 8 bytes |
| 5 | Expiry (`intent.expires_at`) | `u64` big-endian | 8 bytes |
| 6 | Signer kind (`intent.signer.kind`) | `u32` big-endian (`0` = `Direct`, `1` = `Session`, `2` = `Passkey`) | 4 bytes |
| 7 | Session key id (`intent.signer.session_key_id`) | raw bytes (`[0u8; 32]` for non-session signers) | 32 bytes |
| 8 | Signer label (`intent.signer.label`) | Soroban XDR `ScVal::Symbol` (`label.to_xdr(env)`) | variable (`8 + ceil(len / 4) * 4` bytes) |
| 9 | Action system name (`intent.action.system_name`) | Soroban XDR `ScVal::Symbol` (`system_name.to_xdr(env)`) | variable (`8 + ceil(len / 4) * 4` bytes) |
| 10 | Action bytes (`intent.action.data`) | raw bytes | `data.len()` bytes |

Encoding notes for off-chain implementers:

- `Address::to_xdr(env)` encodes `ScVal::Address(ScAddress)`: a 4-byte big-endian `ScValType::Address` discriminant (`0x00000012`) followed by the XDR `ScAddress` union (`ScAddressType::Account = 0` + `PublicKeyTypeEd25519 = 0` + 32-byte Ed25519 key, or `ScAddressType::Contract = 1` + 32-byte contract hash).
- `Symbol::to_xdr(env)` encodes `ScVal::Symbol(ScSymbol)` uniformly for every valid Soroban `Symbol` (0–32 ASCII characters, including symbols longer than 9 characters): a 4-byte big-endian `ScValType::Symbol` discriminant (`0x0000000f`), a 4-byte big-endian byte length, the symbol ASCII bytes, and 0–3 zero padding bytes to a 4-byte boundary.
- Every field before `action.data` is either fixed-width or self-delimiting XDR, preventing concatenation ambiguity between variable-length fields.
- Because `action_hash` commits to `env.current_contract_address()`, `SignedIntent` constructors (`direct`, `session`, `passkey`) and `SignedIntent::recompute_hash` must be called inside a contract invocation context.
- `env.ledger().network_id()` is intentionally not part of the preimage; on Soroban, deployed contract addresses (`C…`) already incorporate the network ID at deployment time.

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
