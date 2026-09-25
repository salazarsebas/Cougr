import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  bytesEqual,
  concatBytes,
  fromHex,
  sha256,
  toBase64Url,
  toHex,
  utf8,
} from '../src/index.ts';

test('hex round-trips and rejects malformed input', () => {
  const bytes = fromHex('00ff10a0');
  assert.deepEqual(Array.from(bytes), [0x00, 0xff, 0x10, 0xa0]);
  assert.equal(toHex(bytes), '00ff10a0');
  assert.throws(() => fromHex('abc'), /hex string is not well formed/);
  assert.throws(() => fromHex('zz'), /hex string is not well formed/);
});

test('concatBytes copies in order', () => {
  const joined = concatBytes(fromHex('0102'), new Uint8Array(), fromHex('03'));
  assert.equal(toHex(joined), '010203');
  assert.equal(bytesEqual(joined, fromHex('010203')), true);
  assert.equal(bytesEqual(joined, fromHex('010204')), false);
  assert.equal(bytesEqual(joined, fromHex('0102')), false);
});

test('base64url has no padding', () => {
  assert.equal(toBase64Url(fromHex('00ff')), 'AP8');
  assert.equal(toBase64Url(fromHex('00ff01')), 'AP8B');
  assert.equal(toBase64Url(new Uint8Array(32)), 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');
  assert.equal(toBase64Url(fromHex('ffffff')), '____');
});

test('sha256 matches the published digest of an empty input', async () => {
  const digest = await sha256(new Uint8Array());
  assert.equal(
    toHex(digest),
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
  );
});

test('utf8 encodes challenge text as bytes', () => {
  assert.equal(toHex(utf8('cougr.test')), '636f7567722e74657374');
});
