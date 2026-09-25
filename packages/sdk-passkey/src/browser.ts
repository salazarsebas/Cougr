/**
 * The thin browser half of `sdk-passkey`.
 *
 * This is the only module that touches `navigator.credentials`. It runs the
 * WebAuthn call and hands the raw result to the pure mapper; every check that
 * can fail lives in `mapper.ts`, so it is covered by node tests.
 */

import { toBase64Url } from './bytes.ts';
import {
  mapAssertion,
  mapRegistration,
  type ClientDataExpectations,
  type MappedAssertion,
  type MappedRegistration,
  type PasskeySigner,
} from './mapper.ts';

export interface WebAuthnContext {
  /** Relying party id, normally the origin hostname. */
  rpId: string;
  /** Exact origin the credential is used from. */
  origin: string;
  /** Display name for the relying party in the account picker. */
  rpName?: string;
}

export type CreatePasskeyOptions = WebAuthnContext & {
  /** Human-readable user name for the credential. */
  userName: string;
  userDisplayName?: string;
  /** 32 random bytes issued by whoever owns the registration. */
  challenge: Uint8Array;
};

export type SignPasskeyOptions = WebAuthnContext & {
  /** The value being endorsed. Goes into `clientDataJSON.challenge` untouched. */
  challenge: Uint8Array;
  /** Credential id returned by `registerPasskey`. */
  credentialId: Uint8Array;
};

export function assertWebAuthnAvailable(): void {
  if (typeof globalThis.navigator === 'undefined' || !globalThis.navigator.credentials) {
    throw new Error('WebAuthn is not available in this environment');
  }
}

/** `navigator.credentials.create` with a challenge the caller owns. */
export async function createPasskeyCredential(
  options: CreatePasskeyOptions,
): Promise<PublicKeyCredential> {
  assertWebAuthnAvailable();

  const userId = new Uint8Array(32);
  globalThis.crypto.getRandomValues(userId);

  const credential = await globalThis.navigator.credentials.create({
    publicKey: {
      challenge: new Uint8Array(options.challenge),
      rp: { id: options.rpId, name: options.rpName ?? options.rpId },
      user: {
        id: userId,
        name: options.userName,
        displayName: options.userDisplayName ?? options.userName,
      },
      pubKeyCredParams: [{ type: 'public-key', alg: -7 }],
      authenticatorSelection: { userVerification: 'preferred' },
      attestation: 'none',
      timeout: 60_000,
    },
  });

  if (!isPublicKeyCredential(credential)) {
    throw new Error('navigator.credentials.create returned no public key credential');
  }
  return credential;
}

/** Register a passkey and return the key bytes `Secp256r1Storage::store` takes. */
export async function registerPasskey(
  options: CreatePasskeyOptions,
): Promise<MappedRegistration> {
  const credential = await createPasskeyCredential(options);
  const response = credential.response as AuthenticatorAttestationResponse;

  return mapRegistration(
    {
      attestationObjectHex: toHexBytes(response.attestationObject),
      clientDataJSON: decodeUtf8(response.clientDataJSON),
    },
    expectations(options, 'webauthn.create'),
  );
}

/**
 * Have the passkey endorse `challenge` and return everything a caller needs to
 * submit it: the `message` and the 64-byte `r || s` signature that
 * `verify_secp256r1` takes.
 *
 * A WebAuthn authenticator never signs a caller-supplied digest - it signs
 * `authenticatorData || sha256(clientDataJSON)`, with the challenge embedded in
 * `clientDataJSON`. So the challenge is what the caller controls, and the
 * authenticator data comes back as part of the message.
 */
export async function signWithPasskey(
  options: SignPasskeyOptions,
): Promise<MappedAssertion> {
  assertWebAuthnAvailable();

  const credential = await globalThis.navigator.credentials.get({
    publicKey: {
      challenge: new Uint8Array(options.challenge),
      rpId: options.rpId,
      allowCredentials: [{ type: 'public-key', id: options.credentialId }],
      userVerification: 'preferred',
      timeout: 60_000,
    },
  });
  if (!isPublicKeyCredential(credential)) {
    throw new Error('navigator.credentials.get returned no public key credential');
  }

  const response = credential.response as AuthenticatorAssertionResponse;
  return mapAssertion(
    {
      authenticatorDataHex: toHexBytes(response.authenticatorData),
      clientDataJSON: decodeUtf8(response.clientDataJSON),
      signatureHex: toHexBytes(response.signature),
    },
    expectations(options, 'webauthn.get'),
  );
}

/**
 * Adapt a registered passkey to the SDK's [`PasskeySigner`] seam: the caller
 * supplies a challenge, the passkey returns the message and `r || s`.
 */
export function createPasskeySigner(options: {
  /** Uncompressed public key from `registerPasskey().publicKey`. */
  publicKey: Uint8Array;
  credentialId: Uint8Array;
  rpId: string;
  origin: string;
}): PasskeySigner {
  return {
    publicKey: options.publicKey,
    async signChallenge(challenge: Uint8Array) {
      const assertion = await signWithPasskey({
        rpId: options.rpId,
        origin: options.origin,
        challenge,
        credentialId: options.credentialId,
      });
      return {
        message: assertion.message,
        messageHex: assertion.messageHex,
        signature: assertion.rawSignature,
        signatureHex: assertion.rawSignatureHex,
      };
    },
  };
}

function expectations(
  options: WebAuthnContext & { challenge: Uint8Array },
  type: 'webauthn.create' | 'webauthn.get',
): ClientDataExpectations {
  return {
    type,
    challenge: toBase64Url(options.challenge),
    origin: options.origin,
    rpId: options.rpId,
  };
}

function isPublicKeyCredential(value: unknown): value is PublicKeyCredential {
  return typeof value === 'object' && value !== null && 'rawId' in value;
}

function toHexBytes(bytes: ArrayBuffer): string {
  return Array.from(new Uint8Array(bytes), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

function decodeUtf8(bytes: ArrayBuffer): string {
  return new TextDecoder().decode(new Uint8Array(bytes));
}
