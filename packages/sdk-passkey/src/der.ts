/**
 * WebAuthn returns ECDSA signatures in ASN.1 DER; Soroban's
 * `env.crypto().secp256r1_verify` expects the 64-byte raw `r || s` form.
 * This is the conversion the contract cannot do for itself.
 */

const SEQUENCE_TAG = 0x30;
const INTEGER_TAG = 0x02;
const COORDINATE_SIZE = 32;
const SIGNATURE_SIZE = 64;

function readLength(bytes: Uint8Array, offset: number): { length: number; next: number } {
  const first = bytes[offset];
  if (first === undefined) {
    throw new Error('DER signature is truncated');
  }
  if ((first & 0x80) === 0) {
    return { length: first, next: offset + 1 };
  }
  const count = first & 0x7f;
  if (count === 0 || count > 2) {
    throw new Error('DER signature uses an unsupported length encoding');
  }
  if (offset + 1 + count > bytes.length) {
    throw new Error('DER signature length is truncated');
  }
  let length = 0;
  for (let i = 0; i < count; i += 1) {
    length = (length << 8) | bytes[offset + 1 + i];
  }
  return { length, next: offset + 1 + count };
}

/**
 * Convert a DER-encoded ECDSA signature into the 64-byte `r || s` form
 * `verify_secp256r1` passes straight to `secp256r1_verify`.
 *
 * Throws on a malformed or out-of-range signature rather than producing bytes
 * the contract would silently reject.
 */
export function derToRawSignature(der: Uint8Array): Uint8Array {
  if (der.length < 8 || der[0] !== SEQUENCE_TAG) {
    throw new Error('DER signature is missing its SEQUENCE tag');
  }

  const sequence = readLength(der, 1);
  if (sequence.next + sequence.length > der.length) {
    throw new Error('DER signature length is inconsistent');
  }

  const raw = new Uint8Array(SIGNATURE_SIZE);
  let cursor = sequence.next;

  for (let integer = 0; integer < 2; integer += 1) {
    if (der[cursor] !== INTEGER_TAG) {
      throw new Error('DER signature is missing an INTEGER');
    }

    const length = readLength(der, cursor + 1);
    let start = length.next;
    let size = length.length;

    if (size === 0 || size > COORDINATE_SIZE + 1) {
      throw new Error('DER integer is out of range');
    }
    if (size > 1 && der[start] === 0x00) {
      start += 1;
      size -= 1;
    }
    if (size === 0 || size > COORDINATE_SIZE) {
      throw new Error('DER integer is out of range');
    }
    if (der.subarray(start, start + size).every((byte) => byte === 0)) {
      throw new Error('DER integer is zero');
    }

    raw.set(der.subarray(start, start + size), integer * COORDINATE_SIZE + (COORDINATE_SIZE - size));
    cursor = length.next + length.length;
  }

  return raw;
}
