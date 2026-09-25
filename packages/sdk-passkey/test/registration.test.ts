import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  assertWebAuthnAvailable,
  createPasskeySigner,
  fromHex,
  registerPasskey,
  signWithPasskey,
  toBase64Url,
  toHex,
  utf8,
} from '../src/index.ts';

interface Vectors {
  rpId: string;
  origin: string;
  challenge: string;
  assertion: {
    authenticatorData: string;
    clientDataJSON: string;
    messageHex: string;
  };
  signatureVectors: { derHex: string; rawHex: string }[];
  registration: {
    attestationObjectHex: string;
    clientDataJSON: string;
    credentialId: string;
    publicKey: string;
  };
}

const vectors = JSON.parse(
  readFileSync(new URL('../vectors/passkey-vectors.json', import.meta.url), 'utf8'),
) as Vectors;

const challengeBytes = fromHex('000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f');
const credentialId = fromHex(vectors.registration.credentialId);
const context = { rpId: vectors.rpId, origin: vectors.origin };

function asArrayBuffer(hex: string): ArrayBuffer {
  const bytes = fromHex(hex);
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.length) as ArrayBuffer;
}

function installCredentials(credentials: unknown): () => void {
  const target = globalThis.navigator as unknown as Record<string, unknown>;
  const had = Object.prototype.hasOwnProperty.call(target, 'credentials');
  const previous = target.credentials;
  target.credentials = credentials;
  return () => {
    if (had) {
      target.credentials = previous;
    } else {
      delete target.credentials;
    }
  };
}

test('registerPasskey wires navigator.credentials.create to the registered key', async () => {
  let seen: { challenge?: Uint8Array; rp?: { id?: string }; alg?: number } = {};
  const restore = installCredentials({
    create: async (options: unknown) => {
      const publicKey = (options as { publicKey: Record<string, unknown> }).publicKey;
      seen = {
        challenge: publicKey.challenge as Uint8Array,
        rp: publicKey.rp as { id?: string },
        alg: (publicKey.pubKeyCredParams as { alg: number }[])[0].alg,
      };
      return {
        rawId: credentialId,
        response: {
          attestationObject: asArrayBuffer(vectors.registration.attestationObjectHex),
          clientDataJSON: asArrayBuffer(toHex(utf8(vectors.registration.clientDataJSON))),
        },
      };
    },
  });

  try {
    const registration = await registerPasskey({
      ...context,
      userName: 'player-one',
      challenge: challengeBytes,
    });

    assert.equal(toHex(seen.challenge as Uint8Array), toHex(challengeBytes));
    assert.equal(seen.rp?.id, vectors.rpId);
    assert.equal(seen.alg, -7);
    assert.equal(registration.publicKeyHex, vectors.registration.publicKey);
    assert.equal(registration.credentialIdHex, vectors.registration.credentialId);
  } finally {
    restore();
  }
});

test('signWithPasskey wires navigator.credentials.get and maps the assertion', async () => {
  let seen: { challenge?: Uint8Array; rpId?: string; credentialId?: Uint8Array } = {};
  const restore = installCredentials({
    get: async (options: unknown) => {
      const publicKey = (options as { publicKey: Record<string, unknown> }).publicKey;
      const allow = publicKey.allowCredentials as { id: Uint8Array }[];
      seen = {
        challenge: publicKey.challenge as Uint8Array,
        rpId: publicKey.rpId as string,
        credentialId: allow[0].id,
      };
      return {
        rawId: credentialId,
        response: {
          authenticatorData: asArrayBuffer(vectors.assertion.authenticatorData),
          clientDataJSON: asArrayBuffer(toHex(utf8(vectors.assertion.clientDataJSON))),
          signature: asArrayBuffer(vectors.signatureVectors[0].derHex),
        },
      };
    },
  });

  try {
    const assertion = await signWithPasskey({ ...context, challenge: challengeBytes, credentialId });

    assert.equal(toBase64Url(seen.challenge as Uint8Array), vectors.challenge);
    assert.equal(seen.rpId, vectors.rpId);
    assert.equal(toHex(seen.credentialId as Uint8Array), vectors.registration.credentialId);
    assert.equal(assertion.messageHex, vectors.assertion.messageHex);
    assert.equal(assertion.rawSignatureHex, vectors.signatureVectors[0].rawHex);
  } finally {
    restore();
  }
});

test('a credential without a navigator is reported, not thrown as a TypeError', () => {
  const restore = installCredentials(undefined);
  try {
    assert.throws(() => assertWebAuthnAvailable(), /WebAuthn is not available/);
  } finally {
    restore();
  }
});

test('createPasskeySigner adapts a passkey to the signer seam', async () => {
  const restore = installCredentials({
    get: async () => ({
      rawId: credentialId,
      response: {
        authenticatorData: asArrayBuffer(vectors.assertion.authenticatorData),
        clientDataJSON: asArrayBuffer(toHex(utf8(vectors.assertion.clientDataJSON))),
        signature: asArrayBuffer(vectors.signatureVectors[1].derHex),
      },
    }),
  });

  try {
    const signer = createPasskeySigner({
      publicKey: fromHex(vectors.registration.publicKey),
      credentialId,
      ...context,
    });

    assert.equal(signer.publicKey.length, 65);
    const signed = await signer.signChallenge(challengeBytes);
    assert.equal(signed.messageHex, vectors.assertion.messageHex);
    assert.equal(signed.signatureHex, vectors.signatureVectors[1].rawHex);
    assert.equal(signed.signature.length, 64);
  } finally {
    restore();
  }
});
