import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  decodeCbor,
  extractEc2PublicKey,
  fromHex,
  parseAttestationObject,
  toHex,
  toUncompressedPublicKey,
  type CborValue,
} from '../src/index.ts';

interface Vectors {
  registration: { attestationObjectHex: string; publicKey: string; credentialId: string };
}

const vectors = JSON.parse(
  readFileSync(new URL('../vectors/passkey-vectors.json', import.meta.url), 'utf8'),
) as Vectors;

function coseKeyOf(entries: [number, CborValue][]): Map<number, CborValue> {
  return new Map(entries);
}

test('the attestation object decodes to fmt none and its authData', () => {
  const parsed = parseAttestationObject(fromHex(vectors.registration.attestationObjectHex));
  assert.equal(parsed.fmt, 'none');
  assert.equal(parsed.authData.length, 148);
});

test('the COSE key in the vector is a P-256 EC2 key and rebuilds the stored form', () => {
  const parsed = parseAttestationObject(fromHex(vectors.registration.attestationObjectHex));
  // The COSE key follows aaguid (16 bytes), the credential id length (2) and the
  // credential id itself, after the rpIdHash/flags/signCount header.
  const credentialId = fromHex(vectors.registration.credentialId);
  const coseOffset = 32 + 1 + 4 + 16 + 2 + credentialId.length;
  const { value } = decodeCbor(parsed.authData, coseOffset);

  assert.ok(value instanceof Map);
  assert.equal(value.get(1), 2, 'kty must be EC2');
  assert.equal(value.get(3), -7, 'alg must be ES256');
  assert.equal(value.get(-1), 1, 'crv must be P-256');

  const uncompressed = toUncompressedPublicKey(extractEc2PublicKey(value));
  assert.equal(uncompressed.length, 65);
  assert.equal(uncompressed[0], 0x04);
  assert.equal(toHex(uncompressed), vectors.registration.publicKey);
});

test('a key that is not a P-256 EC2 key is rejected', () => {
  const p256 = fromHex(vectors.registration.publicKey);
  const x = p256.slice(1, 33);
  const y = p256.slice(33, 65);

  assert.equal(toHex(toUncompressedPublicKey({ x, y })), vectors.registration.publicKey);

  assert.throws(
    () => extractEc2PublicKey(coseKeyOf([[1, 2], [-1, 2], [-2, x], [-3, y]])),
    /not on the P-256 curve/,
  );
  assert.throws(() => extractEc2PublicKey(coseKeyOf([[1, 3], [-1, 1]])), /not an EC2 key/);
  assert.throws(
    () => extractEc2PublicKey(coseKeyOf([[1, 2], [-1, 1], [-2, x.slice(0, 4)], [-3, y]])),
    /invalid x coordinate/,
  );
  assert.throws(
    () => extractEc2PublicKey(coseKeyOf([[1, 2], [-1, 1], [-2, x]])),
    /invalid y coordinate/,
  );
  assert.throws(() => extractEc2PublicKey(1), /not a map/);
});

test('malformed CBOR is rejected rather than guessed at', () => {
  assert.throws(() => decodeCbor(new Uint8Array()), /CBOR item is missing/);
  assert.throws(() => decodeCbor(fromHex('1c')), /argument width 28/);
  assert.throws(() => decodeCbor(fromHex('5820')), /byte string is truncated/);
  assert.throws(() => decodeCbor(fromHex('e1')), /simple value 1/);
  assert.throws(() => decodeCbor(fromHex('5f')), /argument width 31/);
  assert.throws(() => parseAttestationObject(fromHex('01')), /not a CBOR map/);
  assert.throws(() => parseAttestationObject(fromHex('a1616101')), /no format/);
  assert.throws(() => parseAttestationObject(fromHex('a163666d746178')), /no authData/);
});

test('simple values and integers decode to their javascript forms', () => {
  assert.equal(decodeCbor(fromHex('f5')).value, true);
  assert.equal(decodeCbor(fromHex('f4')).value, false);
  assert.equal(decodeCbor(fromHex('f6')).value, null);
  assert.equal(decodeCbor(fromHex('26')).value, -7);
  assert.equal(decodeCbor(fromHex('1a000f4240')).value, 1000000);
});
