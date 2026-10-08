export type BrowserCredentialLike = {
  rawId?: ArrayBuffer | ArrayBufferView | Uint8Array | null;
  response?: {
    getPublicKey?: () => ArrayBuffer | ArrayBufferView | null | undefined;
  } | null;
};

export function toSecp256r1PublicKey(x: Uint8Array, y: Uint8Array): Uint8Array {
  if (x.length !== 32 || y.length !== 32) {
    throw new Error("WebAuthn secp256r1 keys must be 32-byte x and y values.");
  }

  const key = new Uint8Array(65);
  key[0] = 0x04;
  key.set(x, 1);
  key.set(y, 33);
  return key;
}

function toUint8Array(value: ArrayBuffer | ArrayBufferView | null | undefined): Uint8Array {
  if (!value) {
    throw new Error("This browser credential does not expose a public key.");
  }

  if (value instanceof ArrayBuffer) {
    return new Uint8Array(value);
  }

  return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
}

export function mapBrowserCredentialToSecp256r1Bytes(
  credential: BrowserCredentialLike | null | undefined,
): Uint8Array {
  const response = credential?.response;
  if (!response) {
    throw new Error("No browser WebAuthn response was provided.");
  }

  const publicKey = response.getPublicKey?.();
  if (publicKey == null) {
    throw new Error("This browser credential did not expose a secp256r1 public key.");
  }

  const bytes = toUint8Array(publicKey);
  if (bytes.length === 65) {
    return new Uint8Array(bytes);
  }

  if (bytes.length === 64) {
    return toSecp256r1PublicKey(bytes.slice(0, 32), bytes.slice(32, 64));
  }

  throw new Error(`Unexpected WebAuthn public-key length: ${bytes.length}. Expected 65-byte uncompressed secp256r1 key.`);
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}
