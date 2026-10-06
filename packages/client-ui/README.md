# @cougr/client-ui

A small framework-independent toolkit for browser clients that call Cougr
Soroban contracts with Freighter. The package owns the shared wallet, network,
simulation and transaction submission flow. It has no React dependency and does
not include game-specific state or UI.

## Install

Add the local package to a client application's `package.json`:

```json
{
  "dependencies": {
    "@cougr/client-ui": "file:../../packages/client-ui"
  }
}
```

Adjust the relative path for the caller, then run `npm install`.

## Connect and submit a contract call

```ts
import { Networks } from '@stellar/stellar-sdk';
import { createClientUi } from '@cougr/client-ui';

const client = createClientUi({
  contractId: import.meta.env.VITE_CONTRACT_ID,
  rpcUrl: 'https://soroban-testnet.stellar.org',
  networkPassphrase: Networks.TESTNET,
});

const address = await client.requireTestnet();
const transactionHash = await client.invoke(address, 'my_method', []);
```

`requireTestnet()` checks Freighter's selected network before returning its
connected address. `connectedAddress()` reads the address without checking the
network. Call `buildAndSimulate(address, method, args)` when a host needs to
inspect or customize the transaction before signing. It returns the transaction,
simulation result and RPC server. `signAndSubmit(transaction, server)` signs
with Freighter, submits to Soroban RPC and waits for finality. The standalone
`buildAndSimulate`, `signAndSubmit`, and `invoke` exports accept a
`ClientUiConfig` as their first argument for callers that prefer explicit
configuration per call.

The method arguments are Stellar SDK `xdr.ScVal[]` values. The package also
exports `Address` and `nativeToScVal` to help construct them. Invalid setup,
wallet responses, RPC failures, and unsuccessful transactions throw
`ClientError`.

## Framework-independent UI markup

`renderConnectPanel({ address, busy })` returns accessible wallet panel markup
with a `data-client-ui-connect` button. Place it in a host element and delegate
clicks on that attribute to the host's own connect callback. The helper has no
React import and can be used from any browser UI framework or plain DOM code.

`renderMessageBanner({ error, notice })` returns an escaped alert for errors or
status banner for notices. Errors take precedence when both values are passed.

Both helpers return HTML strings; escape any other dynamic values before
inserting them into a page. The helpers escape their own text values.

## Scope

This package handles shared wallet and transaction plumbing. Client apps own
their contract-specific validation, application state, game boards, and
framework components.
