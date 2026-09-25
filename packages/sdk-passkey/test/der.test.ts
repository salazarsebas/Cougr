import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { concatBytes, derToRawSignature, fromHex, toHex } from '../src/index.ts';

interface Vectors {
  signatureVectors: { derHex: string; rawHex: string }[];
}

const vectors = JSON.parse(
  readFileSync(new URL('../vectors/passkey-vectors.json', import.meta.url), 'utf8'),
) as Vectors;

function lengthPrefix(length: number): Uint8Array {
  return length < 0x80 ? new Uint8Array([length]) : new Uint8Array([0x81, length]);
}

function derInteger(value: Uint8Array): Uint8Array {
  return concatBytes(new Uint8Array([0x02]), lengthPrefix(value.length), value);
}

function derSequence(...integers: Uint8Array[]): Uint8Array {
  const body = concatBytes(...integers);
  return concatBytes(new Uint8Array([0x30]), lengthPrefix(body.length), body);
}

test('DER signatures from the vectors become 64 raw bytes', () => {
  for (const vector of vectors.signatureVectors) {
    const raw = derToRawSignature(fromHex(vector.derHex));
    assert.equal(raw.length, 64);
    assert.equal(toHex(raw), vector.rawHex);
  }
});

test('a redundant leading zero on r is stripped, not counted as a coordinate byte', () => {
  // r = 0x00ff... : DER pads a high-bit integer with 0x00, the raw form must not keep it.
  const der = derSequence(derInteger(fromHex('00' + 'ff'.repeat(32))), derInteger(fromHex('01'.repeat(32))));
  assert.equal(toHex(derToRawSignature(der)), 'ff'.repeat(32) + '01'.repeat(32));
});

test('short coordinates are left-padded to 32 bytes', () => {
  const der = derSequence(derInteger(fromHex('0a')), derInteger(fromHex('0b')));
  assert.equal(toHex(derToRawSignature(der)), '00'.repeat(31) + '0a' + '00'.repeat(31) + '0b');
});

test('malformed signatures are rejected', () => {
  const valid = fromHex(vectors.signatureVectors[0].derHex);

  assert.throws(() => derToRawSignature(new Uint8Array()), /SEQUENCE tag/);
  assert.throws(() => derToRawSignature(fromHex('00'.repeat(10))), /SEQUENCE tag/);
  assert.throws(
    () => derToRawSignature(concatBytes(new Uint8Array([0x30, 0x60]), valid.subarray(2, 20))),
    /length is inconsistent/,
  );
  assert.throws(
    () => derToRawSignature(derSequence(derInteger(new Uint8Array(32)), derInteger(fromHex('0b'.repeat(32))))),
    /integer is zero/,
  );
  assert.throws(
    () =>
      derToRawSignature(
        derSequence(derInteger(fromHex('01'.repeat(33))), derInteger(fromHex('0b'.repeat(32)))),
      ),
    /integer is out of range/,
  );
  assert.throws(
    () => derToRawSignature(derSequence(derInteger(fromHex('0a'.repeat(32))))),
    /missing an INTEGER/,
  );
});
