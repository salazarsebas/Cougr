/**
 * cougr-sdk-passkey: WebAuthn registration and assertion mapping for the
 * secp256r1 passkey storage and verification already on chain.
 *
 * The pure mapper is the whole API surface: `mapRegistration` turns a WebAuthn
 * `create` result into a `Secp256r1Key` row, and `mapAssertion` turns a
 * `get` result into the `message` and `r || s` signature `verify_secp256r1`
 * takes. `browser.ts` is a thin `navigator.credentials` wrapper around them.
 */

export {
  assertRpIdHash,
  checkClientData,
  CONTRACT_INVALID_SIGNATURE,
  CONTRACT_SIGNER_NOT_REGISTERED,
  mapAssertion,
  mapRegistration,
  parseAuthenticatorData,
  parseClientDataJSON,
  PasskeyError,
  STORAGE_ENTRYPOINTS,
  toContractKey,
  type AssertionInput,
  type AttestedCredentialData,
  type AuthenticatorData,
  type ClientDataExpectations,
  type ContractSecp256r1Key,
  type MappedAssertion,
  type MappedRegistration,
  type PasskeyErrorCode,
  type PasskeySigner,
  type RegistrationInput,
  type SignedAssertion,
  type WebAuthnClientData,
} from './mapper.ts';

export {
  bytesEqual,
  concatBytes,
  fromHex,
  sha256,
  toBase64Url,
  toHex,
  utf8,
} from './bytes.ts';

export { derToRawSignature } from './der.ts';

export {
  decodeCbor,
  extractEc2PublicKey,
  parseAttestationObject,
  toUncompressedPublicKey,
  type AttestationObject,
  type CoseEc2Key,
  type CborValue,
} from './cbor.ts';

export {
  assertWebAuthnAvailable,
  createPasskeyCredential,
  createPasskeySigner,
  registerPasskey,
  signWithPasskey,
  type CreatePasskeyOptions,
  type SignPasskeyOptions,
  type WebAuthnContext,
} from './browser.ts';
