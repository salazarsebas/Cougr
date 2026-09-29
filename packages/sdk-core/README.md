# cougr-sdk-core

Core simulate-and-submit client for Cougr contracts on Stellar Soroban, with a
versioned discriminated-union error taxonomy covering the full transaction lifecycle.

## Installation

```bash
npm install cougr-sdk-core
```

## Quick start

```ts
import {
  SorobanRpcClient,
  simulateAndSubmit,
  isSdkCoreError,
} from 'cougr-sdk-core';

const rpc = new SorobanRpcClient({ url: 'https://soroban-testnet.stellar.org' });

try {
  const result = await simulateAndSubmit({
    rpc,
    xdr: myUnsignedTransactionXdr,
    sign: async (augmented) => myWallet.sign(augmented),
  });
  console.log('confirmed in ledger', result.ledger);
} catch (err) {
  if (isSdkCoreError(err)) {
    // Branch on kind – no string matching needed.
    switch (err.kind) {
      case 'SimulationError':
        console.error('contract rejected:', err.contractMessage ?? err.simulationError);
        break;
      case 'SubmissionError':
        if (err.retryable) console.warn('network blip – safe to retry');
        else console.error('RPC rejected the transaction');
        break;
      case 'OnchainRejectedError':
        console.error('on-chain FAILED, result XDR:', err.resultXdr);
        break;
      case 'TimeoutError':
        console.warn('still in-flight – poll', err.txHash, 'before retrying');
        break;
    }
  }
}
```

## Error taxonomy — version 1

`TAXONOMY_VERSION = "1"`. Renaming any code below is a **breaking change**.

| Code | Kind | retryable | When it fires |
|---|---|:---:|---|
| `SIMULATION_FAILED` | `SimulationError` | `false` | The Soroban simulation step returned an error. The transaction was never sent. |
| `SUBMISSION_NETWORK_ERROR` | `SubmissionError` | `true` | A transport-level failure (connection reset, DNS, pre-response timeout) prevented the transaction from being sent. The network never saw it. |
| `SUBMISSION_RPC_ERROR` | `SubmissionError` | `false` | The RPC endpoint responded with an HTTP error status or a JSON-RPC error body. The payload was likely malformed or the node rejected it. |
| `ONCHAIN_REJECTED` | `OnchainRejectedError` | `false` | The transaction was included in a ledger but the ledger status is `FAILED`. Retrying the same transaction will always fail. |
| `RESULT_WAIT_TIMEOUT` | `TimeoutError` | `true` | A `SUCCESS` or `FAILED` status was not observed within the configured deadline. The transaction may still be in-flight; poll by `txHash` before retrying. |

### Retry semantics

`err.retryable === true` means it is **safe** to retry the same transaction
unchanged. It does **not** mean retrying will succeed.

`err.retryable === false` means retrying the identical transaction will produce
the same failure. Fix the root cause before trying again.

## Variant-specific fields

### SimulationError

| Field | Type | Description |
|---|---|---|
| `simulationError` | `string \| undefined` | Raw error string from the RPC simulation response. |
| `contractMessage` | `string \| undefined` | Structured reason extracted from the contract's `{ success: false, reason: "…" }` pattern, when present. |

### SubmissionError

| Field | Type | Description |
|---|---|---|
| `httpStatus` | `number \| undefined` | HTTP status code, when the failure was an HTTP response. |
| `rpcCode` | `number \| undefined` | JSON-RPC error code, when the failure was a JSON-RPC error body. |

### OnchainRejectedError

| Field | Type | Description |
|---|---|---|
| `txStatus` | `string` | Status string from the RPC (e.g. `"FAILED"`). |
| `resultXdr` | `string \| undefined` | Raw base-64 XDR result envelope. |

### TimeoutError

| Field | Type | Description |
|---|---|---|
| `txHash` | `string \| undefined` | Hash of the submitted transaction; use it to poll the RPC. |
| `deadlineMs` | `number \| undefined` | The deadline that elapsed, in milliseconds. |

## Type guards

All guards are available as standalone functions and as instance methods on
`SdkCoreError`:

```ts
import {
  isSdkCoreError,
  isSimulationError,
  isSubmissionError,
  isOnchainRejected,
  isTimeoutError,
} from 'cougr-sdk-core';

if (isSimulationError(err)) {
  // err narrowed to SdkCoreError & SimulationError
  console.log(err.contractMessage);
}
```

Or via instance methods:

```ts
if (err instanceof SdkCoreError && err.isTimeout()) {
  console.log(err.txHash);
}
```

## simulateAndSubmit options

| Option | Type | Default | Description |
|---|---|---|---|
| `rpc` | `SorobanRpcClient` | required | The RPC client to use. |
| `xdr` | `string` | required | Base-64 XDR unsigned transaction envelope. |
| `sign` | `(xdr: string) => Promise<string>` | required | Async signer; receives the simulation-augmented XDR. |
| `pollTimeoutMs` | `number` | `30_000` | How long to wait for a ledger result. |
| `pollIntervalMs` | `number` | `2_000` | Interval between poll attempts. |

## Compatibility with #324

`packages/sdk-core` supersedes the simulation-error commitment from #324.
`SIMULATION_FAILED` replaces that package's ad hoc simulation error type; the
`simulationError` and `contractMessage` fields preserve the same information.
No downstream package should need to string-match an error message to branch on
failure kind.
