import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  checkClientData,
  fromHex,
  mapRegistration,
  parseAuthenticatorData,
  parseClientDataJSON,
  PasskeyError,
  STORAGE_ENTRYPOINTS,
  toContractKey,
  toHex,
  utf8,
  type MappedRegistration,
} from '../src/index.ts';

interface Vectors {
  rpId: string;
  origin: string;
  challenge: string;
  assertion: { authenticatorData: string };
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

const context = {
  challenge: vectors.challenge,
  origin: vectors.origin,
  rpId: vectors.rpId,
};

function codeOf(error: unknown): string {
  assert.ok(error instanceof PasskeyError, `expected a PasskeyError, got ${String(error)}`);
  return error.passkeyError;
}

test('parseAuthenticatorData reads the header and detects missing attestation data', () => {
  const parsed = parseAuthenticatorData(fromHex(vectors.assertion.authenticatorData));
  assert.equal(parsed.flags, 0x05);
  assert.equal(parsed.signCount, 1);
  assert.equal(parsed.userPresent, true);
  assert.equal(parsed.userVerified, true);
  assert.equal(parsed.hasAttestedCredentialData, false);
  assert.equal(parsed.hasExtensions, false);
  assert.equal(parsed.attestedCredentialData, undefined);

  assert.throws(() => parseAuthenticatorData(new Uint8Array(36)), /at least 37/);
});

test('parseClientDataJSON rejects anything that is not the three required strings', () => {
  assert.throws(() => parseClientDataJSON('{'), /not valid JSON/);
  assert.throws(() => parseClientDataJSON('[]'), /not an object/);
  assert.throws(() => parseClientDataJSON('{"type":"webauthn.get"}'), /missing a string/);
  assert.equal(parseClientDataJSON('{"type":"a","challenge":"b","origin":"c"}').origin, 'c');
});

test('checkClientData accepts a registrable parent of the origin as rpId', () => {
  const data = parseClientDataJSON(
    JSON.stringify({ type: 'webauthn.get', challenge: 'abc', origin: 'https://cougr.test' }),
  );
  checkClientData(data, {
    type: 'webauthn.get',
    challenge: 'abc',
    origin: 'https://cougr.test',
    rpId: 'test',
  });

  assert.throws(
    () =>
      checkClientData(data, {
        type: 'webauthn.get',
        challenge: 'abc',
        origin: 'https://cougr.test',
        rpId: 'teste',
      }),
    (error: unknown) => codeOf(error) === 'RP_ID_MISMATCH',
  );
  assert.throws(
    () => checkClientData(data, { type: 'webauthn.get', challenge: 'abc', origin: 'not-a-url' }),
    (error: unknown) => codeOf(error) === 'ORIGIN_MISMATCH',
  );
  assert.throws(
    () =>
      checkClientData(data, {
        type: 'webauthn.create',
        challenge: 'abc',
        origin: 'https://cougr.test',
      }),
    (error: unknown) => codeOf(error) === 'WRONG_TYPE',
  );
});

// --- registration edge cases, reached by mutating the pinned vector bytes ---

const AUTH_DATA_FLAGS = 32;
const AUTH_DATA_SIGN_COUNT = 33;
const CREDENTIAL_ID_LENGTH = 16;
const COSE_OFFSET = 32 + 1 + 4 + 16 + 2 + CREDENTIAL_ID_LENGTH;

function findSequence(haystack: Uint8Array, needle: Uint8Array): number {
  for (let i = 0; i + needle.length <= haystack.length; i += 1) {
    if (needle.every((byte, offset) => haystack[i + offset] === byte)) {
      return i;
    }
  }
  throw new Error('needle not found');
}

/** Locate the authData byte string inside the pinned attestation object and edit it. */
function mutateAuthData(mutate: (authData: Uint8Array) => void): string {
  const attestation = fromHex(vectors.registration.attestationObjectHex);
  const label = findSequence(attestation, utf8('authData'));
  const bytesPrefix = label + utf8('authData').length;
  const length = attestation[bytesPrefix + 1];
  const start = bytesPrefix + 2;
  const authData = attestation.slice(start, start + length);
  mutate(authData);
  attestation.set(authData, start);
  return toHex(attestation);
}

test('a registration without attested credential data fails loudly', async () => {
  const attestationObjectHex = mutateAuthData((authData) => {
    authData[AUTH_DATA_FLAGS] = 0x01; // clear the AT flag
  });

  await assert.rejects(
    mapRegistration({ attestationObjectHex, clientDataJSON: vectors.registration.clientDataJSON }, context),
    (error: unknown) => codeOf(error) === 'MISSING_ATTESTATION_DATA',
  );
});

test('a non ES256 algorithm is rejected with the contract error code', async () => {
  const attestationObjectHex = mutateAuthData((authData) => {
    // COSE alg is the third map entry: a5 01 02 03 26 -> flip -7 to -8.
    authData[COSE_OFFSET + 4] = 0x27;
  });

  await assert.rejects(
    mapRegistration({ attestationObjectHex, clientDataJSON: vectors.registration.clientDataJSON }, context),
    (error: unknown) => {
      assert.ok(error instanceof PasskeyError);
      assert.equal(error.contractCode, 22);
      return error.passkeyError === 'UNSUPPORTED_ALGORITHM';
    },
  );
});

test('a non P-256 curve is rejected with the contract error code', async () => {
  const attestationObjectHex = mutateAuthData((authData) => {
    // COSE crv is the fourth map entry: 20 01 -> flip 1 to 2.
    authData[COSE_OFFSET + 6] = 0x02;
  });

  await assert.rejects(
    mapRegistration({ attestationObjectHex, clientDataJSON: vectors.registration.clientDataJSON }, context),
    (error: unknown) => {
      assert.ok(error instanceof PasskeyError);
      assert.equal(error.contractCode, 22);
      return error.passkeyError === 'UNSUPPORTED_CURVE';
    },
  );
});

test('the sign count in a registration is read, not assumed', async () => {
  const attestationObjectHex = mutateAuthData((authData) => {
    authData.set([0x00, 0x00, 0x00, 0x09], AUTH_DATA_SIGN_COUNT);
  });

  const mapped: MappedRegistration = await mapRegistration(
    { attestationObjectHex, clientDataJSON: vectors.registration.clientDataJSON },
    context,
  );
  assert.equal(mapped.signCount, 9);
  assert.equal(mapped.publicKeyHex, vectors.registration.publicKey);
});

test('toContractKey enforces the Soroban symbol length on the label', () => {
  const registration = { publicKeyHex: vectors.registration.publicKey } as MappedRegistration;

  assert.throws(
    () => toContractKey(registration, { label: '', registeredAt: 0 }),
    (error: unknown) => codeOf(error) === 'INVALID_LABEL',
  );
  assert.throws(
    () => toContractKey(registration, { label: 'toolonglabel', registeredAt: 0 }),
    (error: unknown) => codeOf(error) === 'INVALID_LABEL',
  );
  assert.equal(
    toContractKey(registration, { label: 'passkey_2', registeredAt: 7 }).registered_at,
    7,
  );
});

test('the storage entrypoints named in the README are the real ones', () => {
  assert.deepEqual(STORAGE_ENTRYPOINTS, {
    store: 'Secp256r1Storage::store',
    loadAll: 'Secp256r1Storage::load_all',
    find: 'Secp256r1Storage::find_by_label',
    remove: 'Secp256r1Storage::remove',
    verify: 'verify_secp256r1',
  });
});
