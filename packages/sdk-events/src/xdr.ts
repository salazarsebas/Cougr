/**
 * A tiny, dependency-free XDR reader/writer for the subset of `stellar-xdr`
 * that Soroban events use.
 *
 * Soroban RPC returns `topic` entries and `value` as base64-encoded
 * {@link https://github.com/stellar/stellar-xdr XDR} `ScVal` unions. Cougr
 * only needs symbols, unsigned/signed integers, bytes, vectors and maps, but
 * this reader covers the whole `ScVal` union so unknown component payloads
 * never crash a consumer.
 *
 * The point of owning this instead of pulling in `@stellar/stellar-sdk` is
 * that the decoder is small enough to audit and has no transitive
 * dependencies to keep current.
 */

/* ------------------------------------------------------------------ *
 * base64 (works in browsers and Node without Buffer)
 * ------------------------------------------------------------------ */

export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i += 1) {
    binary += String.fromCharCode(bytes[i] as number);
  }
  return btoa(binary);
}

/* ------------------------------------------------------------------ *
 * Reader
 * ------------------------------------------------------------------ */

export class XdrReader {
  readonly #bytes: Uint8Array;
  readonly #view: DataView;
  #offset = 0;

  constructor(input: Uint8Array) {
    this.#bytes = input;
    this.#view = new DataView(input.buffer, input.byteOffset, input.byteLength);
  }

  get offset(): number {
    return this.#offset;
  }

  get remaining(): number {
    return this.#bytes.length - this.#offset;
  }

  readU32(): number {
    this.#need(4);
    const value = this.#view.getUint32(this.#offset, false);
    this.#offset += 4;
    return value;
  }

  readI32(): number {
    this.#need(4);
    const value = this.#view.getInt32(this.#offset, false);
    this.#offset += 4;
    return value;
  }

  readBigUInt(byteLength: number): bigint {
    this.#need(byteLength);
    let value = 0n;
    for (let i = 0; i < byteLength; i += 1) {
      value = (value << 8n) | BigInt(this.#bytes[this.#offset + i] as number);
    }
    this.#offset += byteLength;
    return value;
  }

  readBigInt(byteLength: number): bigint {
    const unsigned = this.readBigUInt(byteLength);
    const signBit = 1n << BigInt(byteLength * 8 - 1);
    return (unsigned & signBit) === 0n ? unsigned : unsigned - (signBit << 1n);
  }

  readFixedOpaque(byteLength: number): Uint8Array {
    this.#need(byteLength);
    const slice = this.#bytes.subarray(this.#offset, this.#offset + byteLength);
    this.#offset += byteLength;
    return slice;
  }

  /** XDR variable-length opaque: u32 length, bytes, zero-padded to 4 bytes. */
  readVarOpaque(): Uint8Array {
    const length = this.readU32();
    const padded = length + ((4 - (length % 4)) % 4);
    this.#need(padded);
    const slice = this.#bytes.subarray(this.#offset, this.#offset + length);
    this.#offset += padded;
    return slice;
  }

  skip(byteLength: number): void {
    this.#need(byteLength);
    this.#offset += byteLength;
  }

  #need(byteLength: number): void {
    if (this.#offset + byteLength > this.#bytes.length) {
      throw new RangeError(
        `XDR read past end of buffer: need ${byteLength} byte(s) at offset ${this.#offset} of ${this.#bytes.length}`,
      );
    }
  }
}

/* ------------------------------------------------------------------ *
 * ScVal
 * ------------------------------------------------------------------ */

export const SCVAL_TYPE = {
  bool: 0,
  void: 1,
  error: 2,
  u32: 3,
  i32: 4,
  u64: 5,
  i64: 6,
  timepoint: 7,
  duration: 8,
  u128: 9,
  i128: 10,
  u256: 11,
  i256: 12,
  bytes: 13,
  string: 14,
  symbol: 15,
  vec: 16,
  map: 17,
  address: 18,
  contractInstance: 19,
  ledgerKeyContractInstance: 20,
  ledgerKeyNonce: 21,
} as const;

export type ScValType = (typeof SCVAL_TYPE)[keyof typeof SCVAL_TYPE];

export type ScVal =
  | { type: 'bool'; value: boolean }
  | { type: 'void' }
  | { type: 'error'; errorType: number; code: number }
  | { type: 'u32'; value: number }
  | { type: 'i32'; value: number }
  | { type: 'u64'; value: bigint }
  | { type: 'i64'; value: bigint }
  | { type: 'timepoint'; value: bigint }
  | { type: 'duration'; value: bigint }
  | { type: 'u128'; value: bigint }
  | { type: 'i128'; value: bigint }
  | { type: 'u256'; value: bigint }
  | { type: 'i256'; value: bigint }
  | { type: 'bytes'; value: Uint8Array }
  | { type: 'string'; value: string }
  | { type: 'symbol'; value: string }
  | { type: 'vec'; value: ScVal[] }
  | { type: 'map'; value: Array<{ key: ScVal; value: ScVal }> }
  | { type: 'address'; value: string }
  | { type: 'contractInstance' }
  | { type: 'ledgerKeyContractInstance' }
  | { type: 'ledgerKeyNonce'; value: bigint }
  | { type: 'unknown'; rawType: number };

const utf8Decoder = new TextDecoder('utf-8', { fatal: false });
const utf8Encoder = new TextEncoder();

export function decodeScValBytes(bytes: Uint8Array): ScVal {
  return readScVal(new XdrReader(bytes));
}

export function decodeScValBase64(base64: string): ScVal {
  return decodeScValBytes(base64ToBytes(base64));
}

function readScVal(reader: XdrReader): ScVal {
  const type = reader.readU32();
  switch (type) {
    case SCVAL_TYPE.bool:
      return { type: 'bool', value: reader.readU32() !== 0 };
    case SCVAL_TYPE.void:
      return { type: 'void' };
    case SCVAL_TYPE.error: {
      const errorType = reader.readU32();
      const code = reader.readU32();
      return { type: 'error', errorType, code };
    }
    case SCVAL_TYPE.u32:
      return { type: 'u32', value: reader.readU32() };
    case SCVAL_TYPE.i32:
      return { type: 'i32', value: reader.readI32() };
    case SCVAL_TYPE.u64:
      return { type: 'u64', value: reader.readBigUInt(8) };
    case SCVAL_TYPE.i64:
      return { type: 'i64', value: reader.readBigInt(8) };
    case SCVAL_TYPE.timepoint:
      return { type: 'timepoint', value: reader.readBigUInt(8) };
    case SCVAL_TYPE.duration:
      return { type: 'duration', value: reader.readBigUInt(8) };
    case SCVAL_TYPE.u128:
      return { type: 'u128', value: reader.readBigUInt(16) };
    case SCVAL_TYPE.i128:
      return { type: 'i128', value: reader.readBigInt(16) };
    case SCVAL_TYPE.u256:
      return { type: 'u256', value: reader.readBigUInt(32) };
    case SCVAL_TYPE.i256:
      return { type: 'i256', value: reader.readBigInt(32) };
    case SCVAL_TYPE.bytes:
      return { type: 'bytes', value: reader.readVarOpaque() };
    case SCVAL_TYPE.string:
      return { type: 'string', value: utf8Decoder.decode(reader.readVarOpaque()) };
    case SCVAL_TYPE.symbol:
      return { type: 'symbol', value: utf8Decoder.decode(reader.readVarOpaque()) };
    case SCVAL_TYPE.vec: {
      if (reader.readU32() === 0) return { type: 'vec', value: [] };
      const count = reader.readU32();
      const value: ScVal[] = [];
      for (let i = 0; i < count; i += 1) value.push(readScVal(reader));
      return { type: 'vec', value };
    }
    case SCVAL_TYPE.map: {
      if (reader.readU32() === 0) return { type: 'map', value: [] };
      const count = reader.readU32();
      const value: Array<{ key: ScVal; value: ScVal }> = [];
      for (let i = 0; i < count; i += 1) {
        const key = readScVal(reader);
        const entryValue = readScVal(reader);
        value.push({ key, value: entryValue });
      }
      return { type: 'map', value };
    }
    case SCVAL_TYPE.address:
      return { type: 'address', value: readAddress(reader) };
    case SCVAL_TYPE.contractInstance:
      return { type: 'contractInstance' };
    case SCVAL_TYPE.ledgerKeyContractInstance:
      return { type: 'ledgerKeyContractInstance' };
    case SCVAL_TYPE.ledgerKeyNonce:
      return { type: 'ledgerKeyNonce', value: reader.readBigInt(8) };
    default:
      return { type: 'unknown', rawType: type };
  }
}

/* ------------------------------------------------------------------ *
 * StrKey (G.../C...) for ScAddress
 * ------------------------------------------------------------------ */

const BASE32_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
const STRKEY_VERSION_ACCOUNT = 6 << 3; // 'G'
const STRKEY_VERSION_CONTRACT = 2 << 3; // 'C'

function readAddress(reader: XdrReader): string {
  const addressType = reader.readU32();
  if (addressType === 0) {
    // SC_ADDRESS_TYPE_ACCOUNT → PublicKey union; only ED25519 is valid today.
    const publicKeyType = reader.readU32();
    const payload = reader.readFixedOpaque(32);
    if (publicKeyType === 0) return encodeStrKey(STRKEY_VERSION_ACCOUNT, payload);
    return `0x${toHex(payload)}`;
  }
  if (addressType === 1) {
    return encodeStrKey(STRKEY_VERSION_CONTRACT, reader.readFixedOpaque(32));
  }
  return `address:${addressType}`;
}

function toHex(bytes: Uint8Array): string {
  let out = '';
  for (let i = 0; i < bytes.length; i += 1) {
    out += (bytes[i] as number).toString(16).padStart(2, '0');
  }
  return out;
}

export function encodeStrKey(versionByte: number, payload: Uint8Array): string {
  const body = new Uint8Array(payload.length + 1);
  body[0] = versionByte;
  body.set(payload, 1);
  const checksum = crc16Xmodem(body);
  const result = new Uint8Array(body.length + 2);
  result.set(body, 0);
  result[body.length] = checksum & 0xff;
  result[body.length + 1] = (checksum >> 8) & 0xff;
  return base32Encode(result);
}

function crc16Xmodem(bytes: Uint8Array): number {
  let crc = 0x0000;
  for (let i = 0; i < bytes.length; i += 1) {
    crc ^= (bytes[i] as number) << 8;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc & 0x8000) !== 0 ? ((crc << 1) ^ 0x1021) & 0xffff : (crc << 1) & 0xffff;
    }
  }
  return crc;
}

function base32Encode(bytes: Uint8Array): string {
  let bits = 0;
  let value = 0;
  let output = '';
  for (let i = 0; i < bytes.length; i += 1) {
    value = (value << 8) | (bytes[i] as number);
    bits += 8;
    while (bits >= 5) {
      output += BASE32_ALPHABET[(value >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits > 0) {
    output += BASE32_ALPHABET[(value << (5 - bits)) & 31];
  }
  return output;
}

/* ------------------------------------------------------------------ *
 * Encoder (used for RPC topic filters)
 * ------------------------------------------------------------------ */

export function encodeSymbolBase64(symbol: string): string {
  const raw = utf8Encoder.encode(symbol);
  if (raw.length > 32) {
    throw new RangeError(`Symbol "${symbol}" is longer than the 32 byte Soroban limit`);
  }
  const padded = raw.length + ((4 - (raw.length % 4)) % 4);
  const out = new Uint8Array(4 + 4 + padded);
  const view = new DataView(out.buffer);
  view.setUint32(0, SCVAL_TYPE.symbol, false);
  view.setUint32(4, raw.length, false);
  out.set(raw, 8);
  return bytesToBase64(out);
}

/* ------------------------------------------------------------------ *
 * Native conversion
 * ------------------------------------------------------------------ */

export type ScValNative =
  | boolean
  | null
  | number
  | bigint
  | string
  | Uint8Array
  | ScValNative[]
  | { [key: string]: ScValNative };

export function scValToNative(value: ScVal): ScValNative {
  switch (value.type) {
    case 'bool':
      return value.value;
    case 'void':
      return null;
    case 'u32':
    case 'i32':
      return value.value;
    case 'u64':
    case 'i64':
    case 'timepoint':
    case 'duration':
    case 'u128':
    case 'i128':
    case 'u256':
    case 'i256':
    case 'ledgerKeyNonce':
      return value.value;
    case 'bytes':
      return value.value;
    case 'string':
    case 'symbol':
    case 'address':
      return value.value;
    case 'vec':
      return value.value.map(scValToNative);
    case 'map': {
      const result: { [key: string]: ScValNative } = {};
      for (const entry of value.value) {
        result[String(scValToNative(entry.key))] = scValToNative(entry.value);
      }
      return result;
    }
    case 'error':
      return `error(${value.errorType},${value.code})`;
    case 'contractInstance':
      return 'contractInstance';
    case 'ledgerKeyContractInstance':
      return 'ledgerKeyContractInstance';
    case 'unknown':
      return `unknown(${value.rawType})`;
  }
}
