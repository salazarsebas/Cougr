/**
 * Minimal CBOR reader for the WebAuthn subset.
 *
 * Only what `navigator.credentials.create` produces is supported: definite-length
 * maps, arrays, text and byte strings, and integers. Indefinite lengths and
 * floating point are rejected rather than guessed at.
 */

export type CborMap = Map<number | string, CborValue>;
export type CborValue = number | boolean | null | Uint8Array | string | CborValue[] | CborMap;

const MAJOR_UINT = 0;
const MAJOR_NEGINT = 1;
const MAJOR_BYTES = 2;
const MAJOR_TEXT = 3;
const MAJOR_ARRAY = 4;
const MAJOR_MAP = 5;
const MAJOR_SIMPLE = 7;

interface Argument {
  value: number;
  next: number;
}

function readArgument(bytes: Uint8Array, offset: number, info: number): Argument {
  if (info < 24) {
    return { value: info, next: offset };
  }
  const widths: Record<number, number> = { 24: 1, 25: 2, 26: 4 };
  const width = widths[info];
  if (width === undefined) {
    throw new Error(`CBOR argument width ${info} is not supported`);
  }
  if (offset + width > bytes.length) {
    throw new Error('CBOR argument is truncated');
  }
  let value = 0;
  for (let i = 0; i < width; i += 1) {
    value = value * 256 + bytes[offset + i];
  }
  return { value, next: offset + width };
}

/** Decode one CBOR item starting at `offset`. */
export function decodeCbor(bytes: Uint8Array, offset = 0): { value: CborValue; next: number } {
  const first = bytes[offset];
  if (first === undefined) {
    throw new Error('CBOR item is missing');
  }

  const major = first >> 5;
  const info = first & 0x1f;
  const argument = readArgument(bytes, offset + 1, info);

  switch (major) {
    case MAJOR_UINT:
      return { value: argument.value, next: argument.next };
    case MAJOR_NEGINT:
      return { value: -1 - argument.value, next: argument.next };
    case MAJOR_BYTES: {
      const end = argument.next + argument.value;
      if (end > bytes.length) {
        throw new Error('CBOR byte string is truncated');
      }
      return { value: bytes.slice(argument.next, end), next: end };
    }
    case MAJOR_TEXT: {
      const end = argument.next + argument.value;
      if (end > bytes.length) {
        throw new Error('CBOR text string is truncated');
      }
      return { value: new TextDecoder().decode(bytes.slice(argument.next, end)), next: end };
    }
    case MAJOR_ARRAY: {
      const items: CborValue[] = [];
      let cursor = argument.next;
      for (let i = 0; i < argument.value; i += 1) {
        const item = decodeCbor(bytes, cursor);
        items.push(item.value);
        cursor = item.next;
      }
      return { value: items, next: cursor };
    }
    case MAJOR_MAP: {
      const map: CborMap = new Map();
      let cursor = argument.next;
      for (let i = 0; i < argument.value; i += 1) {
        const key = decodeCbor(bytes, cursor);
        const value = decodeCbor(bytes, key.next);
        if (typeof key.value !== 'number' && typeof key.value !== 'string') {
          throw new Error('CBOR map keys must be integers or strings');
        }
        map.set(key.value, value.value);
        cursor = value.next;
      }
      return { value: map, next: cursor };
    }
    case MAJOR_SIMPLE: {
      if (info === 20) {
        return { value: false, next: argument.next };
      }
      if (info === 21) {
        return { value: true, next: argument.next };
      }
      if (info === 22) {
        return { value: null, next: argument.next };
      }
      throw new Error(`CBOR simple value ${info} is not supported`);
    }
    default:
      throw new Error(`CBOR major type ${major} is not supported`);
  }
}

export interface AttestationObject {
  fmt: string;
  authData: Uint8Array;
}

/** Split the CBOR attestation object into its format and authenticator data. */
export function parseAttestationObject(bytes: Uint8Array): AttestationObject {
  const { value } = decodeCbor(bytes);
  if (!(value instanceof Map)) {
    throw new Error('attestationObject is not a CBOR map');
  }
  const fmt = value.get('fmt');
  const authData = value.get('authData');
  if (typeof fmt !== 'string') {
    throw new Error('attestationObject has no format');
  }
  if (!(authData instanceof Uint8Array)) {
    throw new Error('attestationObject has no authData');
  }
  return { fmt, authData };
}

export interface CoseEc2Key {
  x: Uint8Array;
  y: Uint8Array;
}

/** Read an EC2 P-256 key out of a COSE_Key structure (RFC 8152). */
export function extractEc2PublicKey(cose: CborValue): CoseEc2Key {
  if (!(cose instanceof Map)) {
    throw new Error('COSE key is not a map');
  }

  const kty = cose.get(1);
  const crv = cose.get(-1);
  const x = cose.get(-2);
  const y = cose.get(-3);

  if (kty !== 2) {
    throw new Error('COSE key is not an EC2 key');
  }
  if (crv !== 1) {
    throw new Error('COSE key is not on the P-256 curve');
  }
  if (!(x instanceof Uint8Array) || x.length !== 32) {
    throw new Error('COSE key has an invalid x coordinate');
  }
  if (!(y instanceof Uint8Array) || y.length !== 32) {
    throw new Error('COSE key has an invalid y coordinate');
  }

  return { x, y };
}

/** Prefix `0x04 || x || y`, the uncompressed form `Secp256r1Key.public_key` stores. */
export function toUncompressedPublicKey(key: CoseEc2Key): Uint8Array {
  const out = new Uint8Array(65);
  out[0] = 0x04;
  out.set(key.x, 1);
  out.set(key.y, 33);
  return out;
}
