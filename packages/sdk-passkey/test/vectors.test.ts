import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  fromHex,
  mapAssertion,
  mapRegistration,
  PasskeyError,
  sha256,
  toContractKey,
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
    clientDataHash: string;
    messageHex: string;
    signCount: number;
  };
  signatureVectors: { derHex: string; rawHex: string }[];
  registration: {
    attestationObjectHex: string;
    clientDataJSON: string;
    credentialId: string;
    publicKey: string;
    signCount: number;
  };
}

const vectors = JSON.parse(
  readFileSync(new URL('../vectors/passkey-vectors.json', import.meta.url), 'utf8'),
) as Vectors;

const context = {
  challenge: vectors.challenge,
  origin: vectors.origin,
  rpId: vectors.rpId,
};

function codeOf(error: unknown): string {
  assert.ok(error instanceof PasskeyError, `expected a PasskeyError, got ${String(error)}`);
  return error.passkeyError;
}

test('the committed assertion message is authenticatorData || sha256(clientDataJSON)', async () => {
  const hash = await sha256(utf8(vectors.assertion.clientDataJSON));
  assert.equal(toHex(hash), vectors.assertion.clientDataHash);
  assert.equal(
    vectors.assertion.authenticatorData + toHex(hash),
    vectors.assertion.messageHex,
  );
});

test('mapAssertion reproduces the vector message and raw signature', async () => {
  const mapped = await mapAssertion(
    {
      authenticatorDataHex: vectors.assertion.authenticatorData,
      clientDataJSON: vectors.assertion.clientDataJSON,
      signatureHex: vectors.signatureVectors[0].derHex,
    },
    context,
  );

  assert.equal(mapped.messageHex, vectors.assertion.messageHex);
  assert.equal(mapped.rawSignatureHex, vectors.signatureVectors[0].rawHex);
  assert.equal(mapped.rawSignature.length, 64);
  assert.equal(mapped.signCount, vectors.assertion.signCount);
  assert.equal(mapped.authenticatorData.userPresent, true);
  assert.equal(mapped.authenticatorData.userVerified, true);
  assert.equal(mapped.authenticatorData.hasAttestedCredentialData, false);
});

test('the rpIdHash in authenticatorData is sha256 of the rpId', async () => {
  const mapped = await mapAssertion(
    {
      authenticatorDataHex: vectors.assertion.authenticatorData,
      clientDataJSON: vectors.assertion.clientDataJSON,
      signatureHex: vectors.signatureVectors[1].derHex,
    },
    context,
  );
  assert.equal(toHex(mapped.authenticatorData.rpIdHash), toHex(await sha256(utf8(vectors.rpId))));
  assert.equal(mapped.rawSignatureHex, vectors.signatureVectors[1].rawHex);
});

test('a wrong challenge is rejected before the signature is mapped', async () => {
  await assert.rejects(
    mapAssertion(
      {
        authenticatorDataHex: vectors.assertion.authenticatorData,
        clientDataJSON: vectors.assertion.clientDataJSON,
        signatureHex: vectors.signatureVectors[0].derHex,
      },
      { ...context, challenge: 'AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh9' },
    ),
    (error: unknown) => codeOf(error) === 'CHALLENGE_MISMATCH',
  );
});

test('a wrong origin is rejected', async () => {
  await assert.rejects(
    mapAssertion(
      {
        authenticatorDataHex: vectors.assertion.authenticatorData,
        clientDataJSON: vectors.assertion.clientDataJSON,
        signatureHex: vectors.signatureVectors[0].derHex,
      },
      { ...context, origin: 'https://cougr.test.evil.example' },
    ),
    (error: unknown) => codeOf(error) === 'ORIGIN_MISMATCH',
  );
});

test('an assertion cannot be replayed as a registration', async () => {
  await assert.rejects(
    mapRegistration(
      {
        attestationObjectHex: vectors.registration.attestationObjectHex,
        clientDataJSON: vectors.assertion.clientDataJSON,
      },
      context,
    ),
    (error: unknown) => codeOf(error) === 'WRONG_TYPE',
  );
});

test('authenticatorData bound to another rpId is rejected', async () => {
  const otherOrigin = 'https://other.test';
  const otherClientData = JSON.stringify({
    type: 'webauthn.get',
    challenge: vectors.challenge,
    origin: otherOrigin,
  });

  await assert.rejects(
    mapAssertion(
      {
        authenticatorDataHex: vectors.assertion.authenticatorData,
        clientDataJSON: otherClientData,
        signatureHex: vectors.signatureVectors[0].derHex,
      },
      { challenge: vectors.challenge, origin: otherOrigin, rpId: 'other.test' },
    ),
    (error: unknown) => codeOf(error) === 'RP_ID_HASH_MISMATCH',
  );
});

test('mapRegistration reproduces the vector key and credential id', async () => {
  const mapped = await mapRegistration(
    {
      attestationObjectHex: vectors.registration.attestationObjectHex,
      clientDataJSON: vectors.registration.clientDataJSON,
    },
    context,
  );

  assert.equal(mapped.fmt, 'none');
  assert.equal(mapped.algorithm, -7);
  assert.equal(mapped.publicKeyHex, vectors.registration.publicKey);
  assert.equal(mapped.credentialIdHex, vectors.registration.credentialId);
  assert.equal(mapped.signCount, vectors.registration.signCount);
  assert.equal(mapped.aaguidHex, '00'.repeat(16));
  assert.equal(mapped.publicKey.length, 65);

  const key = toContractKey(mapped, { label: 'passkey1', registeredAt: 4242 });
  assert.deepEqual(key, {
    public_key: vectors.registration.publicKey,
    label: 'passkey1',
    registered_at: 4242,
  });
  assert.equal(fromHex(key.public_key)[0], 0x04);
});
