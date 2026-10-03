# `@cougr/sdk-core`

The core client library for Cougr, providing an ergonomic path to simulate, assemble, sign, submit, and decode contract invocations on Soroban.

This package doesn't duplicate `stellar-sdk`. It uses it to expose only what games need: an invocation builder, a simulation wrapper, a signer injection point, and result decoders.

## Usage

```typescript
import { CougrClient, Signer } from '@cougr/sdk-core';
import { xdr } from '@stellar/stellar-sdk';

// 1. Implement or inject your signer
class MySigner implements Signer {
  async sign(txXdr: string): Promise<string> {
    // Inject wallet logic or passkeys here
    // return signed base64 xdr
    return "SIGNED_XDR";
  }
}

const client = new CougrClient({
  rpcUrl: 'https://soroban-testnet.stellar.org',
  networkPassphrase: 'Test SDF Network ; September 2015'
});

const mySigner = new MySigner();

// 2. Invoke
const response = await client.invoke(
  {
    contractId: 'C...',
    method: 'do_action',
    args: [xdr.ScVal.scvU32(42)]
  },
  mySigner,
  'G...' // Source account
);

console.log(response.status);
```

## Structure
- `CougrClient`: The main entrypoint. 
- `decodeTurnBasedState`: A fixture decoder mapping a turn-based game state from `ScVal` to a typed TS struct.
