# Cougr web client

This client demonstrates the browser-side part of the passkey flow that connects a WebAuthn credential to the on-chain `tap_battle` secp256r1 verification format.

The app still includes the original turn-based Soroban demo, but it also exposes a `Stellar Drips Wave` passkey proof focused on the browser mapper. The goal is to prove that a browser-generated credential can be mapped into the exact 65-byte `0x04 || x || y` public key format that `verify_secp256r1` accepts, without re-implementing the on-chain verification logic.

## Browser passkey mapper

The key compatibility shim lives in `src/passkey.ts` and enforces the contract format:

- WebAuthn returns the uncompressed P-256 public key in X and Y coordinates.
- The on-chain validator expects a 65-byte SEC-1 point: `0x04 || x || y`.
- The mapper preserves the browser value exactly and rejects unsupported lengths.

A fixture test proves the 65-byte mapping contract without requiring a live browser in CI.

## Configure

Create `clients/web/.env.local`:

    VITE_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

Use the contract instance ID returned by the Stellar CLI after deploying the generated turn-based contract.

## Development

    npm install
    npm run dev
    npm test
    npm run build

The unit tests use fixture state only. They do not connect to Freighter, a live wallet, or a deployed contract.

## Scope

This browser demo is a proof of compatibility, not a replacement for the `sdk-passkey` library or the contract implementation. It intentionally stays limited to the passkey mapper and the local browser generation path.
