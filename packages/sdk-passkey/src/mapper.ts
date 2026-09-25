/**
 * The pure half of `sdk-passkey`: everything between what a WebAuthn call
 * returns and what `cougr_core::accounts::passkey` accepts.
 *
 * Nothing in this file touches `navigator`, `window`, or a network. It takes
 * hex and JSON in and returns hex and bytes out, so the same mapping the browser
 * uses is the one the node test suite pins against the committed vectors.
 *
 * The contract is the spec:
 *
 * - `verify_secp256r1` computes `sha256(message)` and hands the digest to
 *   `env.crypto().secp256r1_verify(public_key, digest, signature)`.
 * - A WebAuthn assertion signs `authenticatorData || sha256(clientDataJSON)`,
 *   so that concatenation is the `message` this module produces.
 * - The signature arrives ASN.1 DER and must become 64 raw bytes `r || s`.
 * - The public key is stored as SEC-1 uncompressed `0x04 || x || y` in
 *   `Secp256r1Key.public_key`, through `Secp256r1Storage::store`.
 */

import { bytesEqual, concatBytes, fromHex, sha256, toHex, utf8 } from './bytes.ts';
import {
  decodeCbor,
  extractEc2PublicKey,
  parseAttestationObject,
  toUncompressedPublicKey,
  type CborValue,
} from './cbor.ts';
import { derToRawSignature } from './der.ts';

/** Account error code for a signature the contract rejects (`InvalidSignature`). */
export const CONTRACT_INVALID_SIGNATURE = 22;
/** Account error code for a label with no stored key (`SignerNotRegistered`). */
export const CONTRACT_SIGNER_NOT_REGISTERED = 44;

export type PasskeyErrorCode =
  | 'MALFORMED_CLIENT_DATA'
  | 'WRONG_TYPE'
  | 'CHALLENGE_MISMATCH'
  | 'ORIGIN_MISMATCH'
  | 'RP_ID_MISMATCH'
  | 'MALFORMED_AUTHENTICATOR_DATA'
  | 'RP_ID_HASH_MISMATCH'
  | 'MISSING_ATTESTATION_DATA'
  | 'MALFORMED_SIGNATURE'
  | 'UNSUPPORTED_ALGORITHM'
  | 'UNSUPPORTED_CURVE'
  | 'UNSUPPORTED_ATTESTATION'
  | 'INVALID_LABEL';

/** A mapping failure, before anything is submitted to the chain. */
export class PasskeyError extends Error {
  readonly passkeyError: PasskeyErrorCode;
  /** The `AccountError` code this failure would surface as on chain, when it has one. */
  readonly contractCode?: number;

  constructor(
    passkeyError: PasskeyErrorCode,
    message: string,
    contractCode?: number,
  ) {
    super(message);
    this.name = 'PasskeyError';
    this.passkeyError = passkeyError;
    this.contractCode = contractCode;
  }
}

/** The subset of `clientDataJSON` the contract's assumptions rest on. */
export interface WebAuthnClientData {
  type: string;
  challenge: string;
  origin: string;
  [key: string]: unknown;
}

export type ClientDataExpectations = {
  /** `webauthn.create` while registering, `webauthn.get` while asserting. */
  type: 'webauthn.create' | 'webauthn.get';
  /** base64url challenge, no padding, issued by whoever owns the action. */
  challenge: string;
  /** Exact origin the credential was created for, e.g. `https://cougr.test`. */
  origin: string;
  /** Relying party id, normally the origin hostname. */
  rpId?: string;
};

export function parseClientDataJSON(clientDataJSON: string): WebAuthnClientData {
  let parsed: unknown;
  try {
    parsed = JSON.parse(clientDataJSON);
  } catch {
    throw new PasskeyError('MALFORMED_CLIENT_DATA', 'clientDataJSON is not valid JSON');
  }
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new PasskeyError('MALFORMED_CLIENT_DATA', 'clientDataJSON is not an object');
  }

  const data = parsed as Record<string, unknown>;
  for (const field of ['type', 'challenge', 'origin'] as const) {
    if (typeof data[field] !== 'string') {
      throw new PasskeyError(
        'MALFORMED_CLIENT_DATA',
        `clientDataJSON is missing a string "${field}"`,
      );
    }
  }
  return data as unknown as WebAuthnClientData;
}

function hostOf(origin: string): string {
  let url: URL;
  try {
    url = new URL(origin);
  } catch {
    throw new PasskeyError('ORIGIN_MISMATCH', `origin "${origin}" is not an absolute URL`);
  }
  return url.hostname;
}

function rpIdMatchesOrigin(rpId: string, origin: string): boolean {
  const host = hostOf(origin);
  return host === rpId || host.endsWith(`.${rpId}`);
}

/** Reject a `clientDataJSON` that does not belong to the challenge being answered. */
export function checkClientData(
  data: WebAuthnClientData,
  expected: ClientDataExpectations,
): void {
  if (data.type !== expected.type) {
    throw new PasskeyError(
      'WRONG_TYPE',
      `clientDataJSON type is "${data.type}", expected "${expected.type}"`,
    );
  }
  if (data.challenge !== expected.challenge) {
    throw new PasskeyError(
      'CHALLENGE_MISMATCH',
      'clientDataJSON challenge does not match the expected challenge',
    );
  }
  if (data.origin !== expected.origin) {
    throw new PasskeyError(
      'ORIGIN_MISMATCH',
      `clientDataJSON origin is "${data.origin}", expected "${expected.origin}"`,
    );
  }
  if (expected.rpId !== undefined && !rpIdMatchesOrigin(expected.rpId, expected.origin)) {
    throw new PasskeyError(
      'RP_ID_MISMATCH',
      `rpId "${expected.rpId}" is not the origin "${expected.origin}" or one of its parents`,
    );
  }
}

const FLAG_USER_PRESENT = 0x01;
const FLAG_USER_VERIFIED = 0x04;
const FLAG_ATTESTED_CREDENTIAL_DATA = 0x40;
const FLAG_EXTENSION_DATA = 0x80;

export interface AttestedCredentialData {
  aaguid: Uint8Array;
  credentialId: Uint8Array;
  cosePublicKey: CborValue;
}

export interface AuthenticatorData {
  rpIdHash: Uint8Array;
  flags: number;
  userPresent: boolean;
  userVerified: boolean;
  hasAttestedCredentialData: boolean;
  hasExtensions: boolean;
  signCount: number;
  attestedCredentialData?: AttestedCredentialData;
  extensions?: CborValue;
}

/** `rpIdHash (32) | flags (1) | signCount (4, big endian) | attestedCredentialData? | extensions?` */
export function parseAuthenticatorData(bytes: Uint8Array): AuthenticatorData {
  if (bytes.length < 37) {
    throw new PasskeyError(
      'MALFORMED_AUTHENTICATOR_DATA',
      `authenticatorData is ${bytes.length} bytes, expected at least 37`,
    );
  }

  const rpIdHash = bytes.slice(0, 32);
  const flags = bytes[32];
  const signCount =
    bytes[33] * 0x1000000 + bytes[34] * 0x10000 + bytes[35] * 0x100 + bytes[36];

  const data: AuthenticatorData = {
    rpIdHash,
    flags,
    userPresent: (flags & FLAG_USER_PRESENT) !== 0,
    userVerified: (flags & FLAG_USER_VERIFIED) !== 0,
    hasAttestedCredentialData: (flags & FLAG_ATTESTED_CREDENTIAL_DATA) !== 0,
    hasExtensions: (flags & FLAG_EXTENSION_DATA) !== 0,
    signCount,
  };

  let cursor = 37;
  if (data.hasAttestedCredentialData) {
    if (bytes.length < cursor + 18) {
      throw new PasskeyError(
        'MALFORMED_AUTHENTICATOR_DATA',
        'attested credential data is truncated',
      );
    }
    const aaguid = bytes.slice(cursor, cursor + 16);
    const credentialIdLength = bytes[cursor + 16] * 0x100 + bytes[cursor + 17];
    const credentialStart = cursor + 18;
    if (bytes.length < credentialStart + credentialIdLength) {
      throw new PasskeyError('MALFORMED_AUTHENTICATOR_DATA', 'credential id is truncated');
    }
    const credentialId = bytes.slice(credentialStart, credentialStart + credentialIdLength);
    const cose = decodeCbor(bytes, credentialStart + credentialIdLength);

    data.attestedCredentialData = { aaguid, credentialId, cosePublicKey: cose.value };
    cursor = cose.next;
  }
  if (data.hasExtensions) {
    const extensions = decodeCbor(bytes, cursor);
    data.extensions = extensions.value;
    cursor = extensions.next;
  }

  return data;
}

/** The rpIdHash ties the credential to the relying party the contract is configured for. */
export async function assertRpIdHash(
  authenticatorData: AuthenticatorData,
  rpId: string,
): Promise<void> {
  const expected = await sha256(utf8(rpId));
  if (!bytesEqual(authenticatorData.rpIdHash, expected)) {
    throw new PasskeyError(
      'RP_ID_HASH_MISMATCH',
      `authenticatorData was not created for rpId "${rpId}"`,
    );
  }
}

export interface AssertionInput {
  /** Raw `authenticatorData` from the credential, hex. */
  authenticatorDataHex: string;
  clientDataJSON: string;
  /** ASN.1 DER signature from the credential, hex. */
  signatureHex: string;
}

export interface MappedAssertion {
  clientData: WebAuthnClientData;
  clientDataHash: Uint8Array;
  authenticatorData: AuthenticatorData;
  /** `authenticatorData || sha256(clientDataJSON)`: the `message` for `verify_secp256r1`. */
  message: Uint8Array;
  messageHex: string;
  /** 64 bytes `r || s`, the `signature` for `verify_secp256r1`. */
  rawSignature: Uint8Array;
  rawSignatureHex: string;
  signCount: number;
}

/**
 * Map a WebAuthn assertion into the two arguments `verify_secp256r1` takes,
 * after checking the challenge, origin, and rpId it was made for.
 */
export async function mapAssertion(
  input: AssertionInput,
  expected: Omit<ClientDataExpectations, 'type'>,
): Promise<MappedAssertion> {
  const clientData = parseClientDataJSON(input.clientDataJSON);
  checkClientData(clientData, { ...expected, type: 'webauthn.get' });

  const rawAuthData = fromHex(input.authenticatorDataHex);
  const authenticatorData = parseAuthenticatorData(rawAuthData);
  await assertRpIdHash(authenticatorData, expected.rpId ?? hostOf(expected.origin));

  const clientDataHash = await sha256(utf8(input.clientDataJSON));
  const message = concatBytes(rawAuthData, clientDataHash);

  const rawSignature = derToRawSignature(fromHex(input.signatureHex));

  return {
    clientData,
    clientDataHash,
    authenticatorData,
    message,
    messageHex: toHex(message),
    rawSignature,
    rawSignatureHex: toHex(rawSignature),
    signCount: authenticatorData.signCount,
  };
}

export interface RegistrationInput {
  /** CBOR `attestationObject` from `navigator.credentials.create`, hex. */
  attestationObjectHex: string;
  clientDataJSON: string;
}

export interface MappedRegistration {
  /** Attestation statement format, always `none` for `navigator.credentials.create` without attestation. */
  fmt: string;
  aaguid: Uint8Array;
  aaguidHex: string;
  credentialId: Uint8Array;
  credentialIdHex: string;
  /** SEC-1 uncompressed key `0x04 || x || y`, the value of `Secp256r1Key.public_key`. */
  publicKey: Uint8Array;
  publicKeyHex: string;
  /** COSE algorithm, `-7` (ES256) for the curve the contract verifies. */
  algorithm: number;
  signCount: number;
  userVerified: boolean;
}

/** Map a WebAuthn registration into the `Secp256r1Key` the contract stores. */
export async function mapRegistration(
  input: RegistrationInput,
  expected: ClientDataExpectations,
): Promise<MappedRegistration> {
  const clientData = parseClientDataJSON(input.clientDataJSON);
  checkClientData(clientData, { ...expected, type: 'webauthn.create' });

  const attestation = parseAttestationObject(fromHex(input.attestationObjectHex));
  const authenticatorData = parseAuthenticatorData(attestation.authData);
  await assertRpIdHash(authenticatorData, expected.rpId ?? hostOf(expected.origin));

  const attested = authenticatorData.attestedCredentialData;
  if (attested === undefined) {
    throw new PasskeyError(
      'MISSING_ATTESTATION_DATA',
      'registration has no attested credential data (AT flag is unset)',
    );
  }

  const cose = attested.cosePublicKey;
  const algorithm = cose instanceof Map ? cose.get(3) : undefined;
  if (algorithm !== -7) {
    throw new PasskeyError(
      'UNSUPPORTED_ALGORITHM',
      `registration used COSE algorithm ${String(algorithm)}, expected -7 (ES256)`,
      CONTRACT_INVALID_SIGNATURE,
    );
  }

  let publicKey: Uint8Array;
  try {
    publicKey = toUncompressedPublicKey(extractEc2PublicKey(cose));
  } catch (error) {
    throw new PasskeyError(
      'UNSUPPORTED_CURVE',
      error instanceof Error ? error.message : 'registration key is not a P-256 EC2 key',
      CONTRACT_INVALID_SIGNATURE,
    );
  }

  return {
    fmt: attestation.fmt,
    aaguid: attested.aaguid,
    aaguidHex: toHex(attested.aaguid),
    credentialId: attested.credentialId,
    credentialIdHex: toHex(attested.credentialId),
    publicKey,
    publicKeyHex: toHex(publicKey),
    algorithm,
    signCount: authenticatorData.signCount,
    userVerified: authenticatorData.userVerified,
  };
}

/**
 * The `Secp256r1Key` row `Secp256r1Storage::store(env, account, key)` expects.
 * `label` becomes a Soroban `Symbol`, hence the `symbol_short!` length limit.
 */
export interface ContractSecp256r1Key {
  public_key: string;
  label: string;
  registered_at: number;
}

export const STORAGE_ENTRYPOINTS = {
  store: 'Secp256r1Storage::store',
  loadAll: 'Secp256r1Storage::load_all',
  find: 'Secp256r1Storage::find_by_label',
  remove: 'Secp256r1Storage::remove',
  verify: 'verify_secp256r1',
} as const;

export function toContractKey(
  registration: MappedRegistration,
  options: { label: string; registeredAt: number },
): ContractSecp256r1Key {
  if (options.label.length === 0 || options.label.length > 9) {
    throw new PasskeyError(
      'INVALID_LABEL',
      'label must be 1-9 characters because Secp256r1Key.label is a Soroban symbol',
    );
  }
  return {
    public_key: registration.publicKeyHex,
    label: options.label,
    registered_at: options.registeredAt,
  };
}

/**
 * What a passkey returns for a challenge: the preimage and the signature, in the
 * exact shapes `verify_secp256r1(env, public_key, &message, &signature)` takes.
 */
export interface SignedAssertion {
  /** `authenticatorData || sha256(clientDataJSON)`. */
  message: Uint8Array;
  messageHex: string;
  /** 64 bytes `r || s`, never DER. */
  signature: Uint8Array;
  signatureHex: string;
}

/**
 * The smallest signer the SDK needs: something that holds the uncompressed key
 * and can endorse a caller-supplied challenge.
 *
 * `cougr-core` names the same seam `AccountSigner`; a chain to this interface is
 * in `browser.ts` (`createPasskeySigner`).
 *
 * Note the contract-side contract: it verifies `sha256(message)` against the
 * stored key, and an authenticator only signs the WebAuthn assertion message.
 * A caller wiring a passkey into a signed intent therefore has to submit the
 * `message` this returns, not the raw action hash, because the authenticator
 * data is part of what was signed.
 */
export interface PasskeySigner {
  /** SEC-1 uncompressed public key, 65 bytes. */
  readonly publicKey: Uint8Array;
  signChallenge(challenge: Uint8Array): Promise<SignedAssertion>;
}
