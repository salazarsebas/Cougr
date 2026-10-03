import { describe, expect, it } from "vitest";
import { mapBrowserCredentialToSecp256r1Bytes, toSecp256r1PublicKey } from "./passkey";

describe("browser passkey mapping", () => {
  it("wraps the uncompressed public key in the exact 65-byte format expected by verify_secp256r1", () => {
    const x = new Uint8Array(32).fill(0x11);
    const y = new Uint8Array(32).fill(0x22);

    const mapped = toSecp256r1PublicKey(x, y);

    expect(mapped).toHaveLength(65);
    expect(Array.from(mapped.slice(0, 1))).toEqual([0x04]);
    expect(Array.from(mapped.slice(1, 33))).toEqual(Array.from(x));
    expect(Array.from(mapped.slice(33))).toEqual(Array.from(y));
  });

  it("accepts a browser WebAuthn public key buffer without re-encoding the x/y values", () => {
    const key = new Uint8Array(65);
    key[0] = 0x04;
    key.set(new Uint8Array(32).fill(0x33), 1);
    key.set(new Uint8Array(32).fill(0x44), 33);

    const result = mapBrowserCredentialToSecp256r1Bytes({
      rawId: new Uint8Array(0),
      response: {
        getPublicKey: () => key.buffer
      }
    });

    expect(Array.from(result)).toEqual(Array.from(key));
  });
});
