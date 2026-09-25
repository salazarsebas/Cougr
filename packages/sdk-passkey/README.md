# cougr-sdk-passkey

WebAuthn registration and assertion mapping for the secp256r1 passkeys Cougr already
verifies on chain. This package adds the off-chain half — `navigator.credentials` and the byte
mapping a browser needs — against the contract that already exists
([`src/accounts/secp256r1_auth.rs`](../../src/accounts/secp256r1_auth.rs),
`accounts::passkey`, `Secp256r1Storage`, `verify_secp256r1`). It does not add a curve, a
contract, or a custodial fallback.

The contract is the spec. `verify_secp256r1` computes `sha256(message)` and passes the digest to
`env.crypto().secp256r1_verify`, so this package produces the `message` and the 64-byte `r || s`
signature that call takes, and nothing else.

## What is in here

| Path | Role |
|---|---|
| `src/mapper.ts` | Pure mapping: `mapRegistration`, `mapAssertion`, `parseAuthenticatorData`, `toContractKey`, `PasskeyError` |
| `src/cbor.ts` | The CBOR subset WebAuthn uses, plus COSE EC2 key extraction |
| `src/der.ts` | ASN.1 DER to 64 raw bytes `r || s` |
| `src/bytes.ts` | Hex, base64url, SHA-256, byte helpers |
| `src/browser.ts` | The only module that touches `navigator.credentials` |
| `src/index.ts` | Public entry point |
| `vectors/passkey-vectors.json` | Cross-language vectors, pinned by the Rust test and asserted here |
| `test/*.test.ts` | Mapper, CBOR, DER, vector, and browser-wiring tests |

There is no build step. Node 22.6+ runs the TypeScript sources directly, so `npm test` needs no
toolchain and no install.

```bash
npm test          # node --test test/bytes.test.ts test/der.test.ts test/cbor.test.ts \
                  #   test/mapper.test.ts test/registration.test.ts test/vectors.test.ts
```

## The three mappings

A WebAuthn assertion is not the shape the contract verifies. `mapAssertion` bridges the three
gaps, after checking the challenge, origin, and rpId:

| WebAuthn | Contract | Where |
|---|---|---|
| `authenticatorData` (37+ bytes) | `verify_secp256r1(env, key, &message, &signature)` | `mapAssertion().message` |
| `clientDataJSON` (UTF-8 JSON) | `sha256(clientDataJSON)` appended to the message | `mapAssertion().clientDataHash` |
| `signature` (ASN.1 DER) | `BytesN<64>` = `r || s`, 32 bytes each, big-endian | `mapAssertion().rawSignature` |
| COSE EC2 key in `attestationObject` | `Secp256r1Key.public_key: BytesN<65>` = `0x04 || x || y` | `mapRegistration().publicKey` |

The message is exactly `authenticatorData || sha256(clientDataJSON)`; the contract hashes that
concatenation once more with `env.crypto().sha256`, which is the digest
`secp256r1_verify` receives. That is the only shape where the two sides agree — a DER signature
cannot be submitted as-is, because `secp256r1_verify` takes fixed-width coordinates.

## Storage entrypoints used

Registration is written through the existing storage API — no new storage, no new key type:

| Entrypoint | Use here |
|---|---|
| `Secp256r1Storage::store(env, account, &Secp256r1Key)` | What `toContractKey()` shapes a row for |
| `Secp256r1Storage::load_all(env, account)` / `find_by_label(env, account, label)` | Reading back a registered passkey |
| `Secp256r1Storage::remove(env, account, label)` | Revocation, unchanged |
| `verify_secp256r1(env, public_key, message, signature)` | What `mapAssertion()` prepares arguments for |

`Secp256r1Key.label` is a Soroban `Symbol`, so `toContractKey` rejects a label outside 1-9
characters instead of letting `symbol_short!` panic later. `registered_at` is a ledger timestamp in
seconds, supplied by the caller.

```ts
import { mapRegistration, toContractKey } from 'cougr-sdk-passkey';

const registration = await mapRegistration(
  { attestationObjectHex, clientDataJSON },
  { challenge, origin: 'https://cougr.test', rpId: 'cougr.test' },
);

const key = toContractKey(registration, { label: 'passkey1', registeredAt: ledgerTimestamp });
// -> { public_key: "04…", label: "passkey1", registered_at: 1699999999 }
```

## rpId, origin, and challenge assumptions

WebAuthn binds a credential to a relying party and an origin, and Cougr does not get to relax that.
The package makes those assumptions explicit and checks them before producing bytes:

- **`rpId`** is normally the origin hostname (`cougr.test` for `https://cougr.test`). The
  `authenticatorData` `rpIdHash` must equal `sha256(rpId)`, and the origin the credential was made
  for must be the `rpId` itself or one of its subdomains. A credential minted for another relying
  party is rejected with `RP_ID_HASH_MISMATCH` or `RP_ID_MISMATCH`.
- **`origin`** is compared to `clientDataJSON.origin` byte for byte, including scheme. No suffix
  matching, so `https://cougr.test.evil.example` is not `https://cougr.test`.
- **`challenge`** must equal `clientDataJSON.challenge` exactly, and is compared base64url with no
  padding. The caller owns challenge generation and single-use storage; this package does not
  invent a challenge source.
- **`type`** must be `webauthn.create` while registering and `webauthn.get` while asserting, so an
  assertion cannot be replayed as a registration.

## The signer interface

The package does not fork the RPC stack, and it does not pretend a passkey can sign an arbitrary
digest. `PasskeySigner` is the seam:

```ts
import { createPasskeySigner } from 'cougr-sdk-passkey';

const signer = createPasskeySigner({ publicKey, credentialId, rpId: 'cougr.test', origin: 'https://cougr.test' });
const signed = await signer.signChallenge(challenge);
// signed.message = authenticatorData || sha256(clientDataJSON)
// signed.signature = 64 raw bytes r || s
```

A WebAuthn authenticator only signs `authenticatorData || sha256(clientDataJSON)`, with the
challenge embedded in `clientDataJSON`. So a caller wiring a passkey into a signed intent submits
`signed.message` as the `message` for `verify_secp256r1` — not the raw action hash, because the
authenticator data is part of what was signed. `cougr_core::accounts::Secp256r1PasskeySigner`
currently passes `intent.action_hash.to_bytes()`; a caller using that path has to set the action
hash to the mapped message above, because the contract's check is the one that counts. This package
does not loosen the contract check to hide that difference.

## Cross-language vectors

`vectors/passkey-vectors.json` is the shared artifact:

- `tests/passkey_vectors.rs` re-derives the assertion message with the SDK's own
  `env.crypto().sha256`, checks the `rpIdHash`, converts both DER vectors, walks the pinned
  `attestationObject` to its COSE key, and round-trips the mapped key through
  `Secp256r1Storage::store` / `load_all`.
- `test/vectors.test.ts` asserts the same JSON against the TypeScript mapper, including the
  negative cases (wrong challenge, wrong origin, wrong rpId, an assertion replayed as a
  registration).

The registration vector uses an attestation format of `none` (the shape
`navigator.credentials.create` returns without attestation) and a test key, not a real credential.
Attestation statement verification is out of scope: `fmt` is reported, not validated, because the
trust decision belongs to whoever accepts the published key.

## Known limitations

- **ES256 only.** The COSE algorithm must be `-7` and the curve P-256, matching
  `env.crypto().secp256r1_verify`. Anything else is rejected with
  `UNSUPPORTED_ALGORITHM` / `UNSUPPORTED_CURVE` (contract code 22).
- **No credential storage.** The package never persists a credential, a resident key, or any seed
  material; there is none in the repository. Storage and challenge lifetime are the caller's job.
- **No replay protection.** A passkey assertion is not a session. Expiry, nonces, and budgets stay
  in `src/accounts` (`SessionBuilder`, `SessionPolicy`); this package is the signer those flows can
  use, not a second policy engine.
- **No RPC.** Anything that submits a transaction takes a function argument, as in
  `packages/sdk-session`, until `sdk-core` lands.
- The browser wrapper is a thin `navigator.credentials` call and is tested with a fake credential
  object; real-device behaviour (platform authenticator prompts, user cancellation) is not covered
  by the node suite.
