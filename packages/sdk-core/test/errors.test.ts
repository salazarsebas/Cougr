/**
 * Tests for the sdk-core error taxonomy and simulate-and-submit path.
 *
 * Each test forces one failure path against a mocked RPC response and asserts
 * the resulting SdkCoreError shape (kind, code, retryable, and variant-specific
 * fields).
 */

import assert from 'node:assert/strict';
import test from 'node:test';

import {
  SdkCoreError,
  SorobanRpcClient,
  simulateAndSubmit,
  isSdkCoreError,
  isSimulationError,
  isSubmissionError,
  isOnchainRejected,
  isTimeoutError,
  extractContractReason,
  TAXONOMY_VERSION,
  type FetchLike,
  type SimulateTransactionResult,
  type SendTransactionResult,
  type GetTransactionResult,
} from '../src/index.ts';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Build a FetchLike that returns a sequence of JSON bodies. */
function mockFetch(responses: Array<{ ok?: boolean; status?: number; body: unknown }>): FetchLike {
  let idx = 0;
  return async (_url, _init) => {
    const resp = responses[idx++] ?? responses.at(-1)!;
    const ok = resp.ok ?? true;
    const status = resp.status ?? 200;
    return {
      ok,
      status,
      async json() {
        return resp.body;
      },
      async text() {
        return JSON.stringify(resp.body);
      },
    };
  };
}

function rpcOk(result: unknown) {
  return { body: { jsonrpc: '2.0', id: 1, result } };
}

function rpcErr(code: number, message: string) {
  return { body: { jsonrpc: '2.0', id: 1, error: { code, message } } };
}

const NOOP_SIGN = async (xdr: string) => xdr;
const BASE_XDR = 'AAAA'; // placeholder

// Simulation success body
const SIM_OK: SimulateTransactionResult = {
  results: [{ xdr: 'resultXdr' }],
  transactionData: 'footprint',
  minResourceFee: '100',
};
// sendTransaction pending
const SEND_PENDING: SendTransactionResult = { hash: 'abc123', status: 'PENDING' };
// getTransaction success
const GET_SUCCESS: GetTransactionResult = {
  status: 'SUCCESS',
  resultXdr: 'resultXdrBase64==',
  ledger: 42,
};

// ---------------------------------------------------------------------------
// TAXONOMY_VERSION
// ---------------------------------------------------------------------------

test('TAXONOMY_VERSION is the string "1"', () => {
  assert.equal(TAXONOMY_VERSION, '1');
});

// ---------------------------------------------------------------------------
// Static factories and shape
// ---------------------------------------------------------------------------

test('SdkCoreError.simulationFailed produces correct shape', () => {
  const err = SdkCoreError.simulationFailed({
    message: 'sim failed',
    simulationError: 'raw error',
    contractMessage: 'not your turn',
  });
  assert.ok(err instanceof SdkCoreError);
  assert.ok(err instanceof Error);
  assert.equal(err.kind, 'SimulationError');
  assert.equal(err.code, 'SIMULATION_FAILED');
  assert.equal(err.retryable, false);
  assert.equal(err.simulationError, 'raw error');
  assert.equal(err.contractMessage, 'not your turn');
  assert.equal(err.message, 'sim failed');
});

test('SdkCoreError.networkError produces correct shape', () => {
  const cause = new TypeError('connection reset');
  const err = SdkCoreError.networkError({ message: 'net fail', cause });
  assert.equal(err.kind, 'SubmissionError');
  assert.equal(err.code, 'SUBMISSION_NETWORK_ERROR');
  assert.equal(err.retryable, true);
  assert.equal(err.cause, cause);
});

test('SdkCoreError.rpcError produces correct shape', () => {
  const err = SdkCoreError.rpcError({ message: 'rpc fail', httpStatus: 503, rpcCode: -32602 });
  assert.equal(err.kind, 'SubmissionError');
  assert.equal(err.code, 'SUBMISSION_RPC_ERROR');
  assert.equal(err.retryable, false);
  assert.equal(err.httpStatus, 503);
  assert.equal(err.rpcCode, -32602);
});

test('SdkCoreError.onchainRejected produces correct shape', () => {
  const err = SdkCoreError.onchainRejected({
    message: 'tx FAILED',
    txStatus: 'FAILED',
    resultXdr: 'xdr==',
  });
  assert.equal(err.kind, 'OnchainRejectedError');
  assert.equal(err.code, 'ONCHAIN_REJECTED');
  assert.equal(err.retryable, false);
  assert.equal(err.txStatus, 'FAILED');
  assert.equal(err.resultXdr, 'xdr==');
});

test('SdkCoreError.resultWaitTimeout produces correct shape', () => {
  const err = SdkCoreError.resultWaitTimeout({
    message: 'timeout',
    txHash: 'deadbeef',
    deadlineMs: 30_000,
  });
  assert.equal(err.kind, 'TimeoutError');
  assert.equal(err.code, 'RESULT_WAIT_TIMEOUT');
  assert.equal(err.retryable, true);
  assert.equal(err.txHash, 'deadbeef');
  assert.equal(err.deadlineMs, 30_000);
});

// ---------------------------------------------------------------------------
// Type guards
// ---------------------------------------------------------------------------

test('isSdkCoreError returns true for SdkCoreError and false for plain Error', () => {
  assert.ok(isSdkCoreError(SdkCoreError.simulationFailed({ message: 'x' })));
  assert.ok(!isSdkCoreError(new Error('plain')));
  assert.ok(!isSdkCoreError('string'));
  assert.ok(!isSdkCoreError(null));
});

test('isSimulationError narrows correctly', () => {
  const sim = SdkCoreError.simulationFailed({ message: 'x' });
  const net = SdkCoreError.networkError({ message: 'x' });
  assert.ok(isSimulationError(sim));
  assert.ok(!isSimulationError(net));
});

test('isSubmissionError narrows correctly', () => {
  const net = SdkCoreError.networkError({ message: 'x' });
  const rpc = SdkCoreError.rpcError({ message: 'x' });
  const sim = SdkCoreError.simulationFailed({ message: 'x' });
  assert.ok(isSubmissionError(net));
  assert.ok(isSubmissionError(rpc));
  assert.ok(!isSubmissionError(sim));
});

test('isOnchainRejected narrows correctly', () => {
  const rej = SdkCoreError.onchainRejected({ message: 'x', txStatus: 'FAILED' });
  const tim = SdkCoreError.resultWaitTimeout({ message: 'x' });
  assert.ok(isOnchainRejected(rej));
  assert.ok(!isOnchainRejected(tim));
});

test('isTimeoutError narrows correctly', () => {
  const tim = SdkCoreError.resultWaitTimeout({ message: 'x' });
  const rej = SdkCoreError.onchainRejected({ message: 'x', txStatus: 'FAILED' });
  assert.ok(isTimeoutError(tim));
  assert.ok(!isTimeoutError(rej));
});

// Instance method narrowing
test('instance method isSimulationError narrows', () => {
  const err = SdkCoreError.simulationFailed({ message: 'x', contractMessage: 'oops' });
  if (err.isSimulationError()) {
    // TypeScript should know contractMessage is accessible here
    assert.equal(err.contractMessage, 'oops');
  } else {
    assert.fail('Expected isSimulationError() to return true');
  }
});

// ---------------------------------------------------------------------------
// extractContractReason
// ---------------------------------------------------------------------------

test('extractContractReason finds quoted reason values', () => {
  assert.equal(extractContractReason('reason: "not your turn"'), 'not your turn');
  assert.equal(extractContractReason('"reason":"invalid move"'), 'invalid move');
  assert.equal(extractContractReason("reason: 'game over'"), 'game over');
});

test('extractContractReason finds unquoted reason values', () => {
  assert.equal(extractContractReason('reason = game_over'), 'game_over');
});

test('extractContractReason returns undefined when no reason present', () => {
  assert.equal(extractContractReason('some random error string'), undefined);
  assert.equal(extractContractReason(''), undefined);
});

// ---------------------------------------------------------------------------
// SorobanRpcClient – network errors
// ---------------------------------------------------------------------------

test('SorobanRpcClient maps a thrown fetch error to SUBMISSION_NETWORK_ERROR', async () => {
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: async () => {
      throw new TypeError('connection refused');
    },
  });

  await assert.rejects(
    () => client.simulateTransaction(BASE_XDR),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SUBMISSION_NETWORK_ERROR');
      assert.equal(err.retryable, true);
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// SorobanRpcClient – HTTP error
// ---------------------------------------------------------------------------

test('SorobanRpcClient maps an HTTP 503 to SUBMISSION_RPC_ERROR (not retryable)', async () => {
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([{ ok: false, status: 503, body: 'service unavailable' }]),
  });

  await assert.rejects(
    () => client.simulateTransaction(BASE_XDR),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SUBMISSION_RPC_ERROR');
      assert.equal(err.retryable, false);
      assert.equal((err as SdkCoreError).httpStatus, 503);
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// SorobanRpcClient – JSON-RPC error body
// ---------------------------------------------------------------------------

test('SorobanRpcClient maps a JSON-RPC error body to SUBMISSION_RPC_ERROR', async () => {
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([rpcErr(-32602, 'invalid params')]),
  });

  await assert.rejects(
    () => client.simulateTransaction(BASE_XDR),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SUBMISSION_RPC_ERROR');
      assert.equal(err.retryable, false);
      assert.equal((err as SdkCoreError).rpcCode, -32602);
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// simulateAndSubmit – simulation failure
// ---------------------------------------------------------------------------

test('simulateAndSubmit maps a simulation error response to SIMULATION_FAILED', async () => {
  const simError: SimulateTransactionResult = { error: 'HostError: contract error' };
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([rpcOk(simError)]),
  });

  await assert.rejects(
    () => simulateAndSubmit({ rpc: client, xdr: BASE_XDR, sign: NOOP_SIGN }),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SIMULATION_FAILED');
      assert.equal(err.retryable, false);
      assert.equal((err as SdkCoreError).simulationError, 'HostError: contract error');
      return true;
    },
  );
});

test('simulateAndSubmit extracts contractMessage from simulation error', async () => {
  const simError: SimulateTransactionResult = {
    error: 'contract panicked: reason: "not your turn"',
  };
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([rpcOk(simError)]),
  });

  await assert.rejects(
    () => simulateAndSubmit({ rpc: client, xdr: BASE_XDR, sign: NOOP_SIGN }),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SIMULATION_FAILED');
      assert.equal((err as SdkCoreError).contractMessage, 'not your turn');
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// simulateAndSubmit – submission network error (thrown by fetch)
// ---------------------------------------------------------------------------

test('simulateAndSubmit maps fetch throw during sendTransaction to SUBMISSION_NETWORK_ERROR', async () => {
  let call = 0;
  const fetchLike: FetchLike = async (_url, _init) => {
    call++;
    if (call === 1) {
      // Simulate succeeds
      return {
        ok: true,
        status: 200,
        async json() {
          return { jsonrpc: '2.0', id: 1, result: SIM_OK };
        },
        async text() {
          return '';
        },
      };
    }
    // sendTransaction throws
    throw new TypeError('connection reset by peer');
  };

  const client = new SorobanRpcClient({ url: 'https://rpc.example', fetch: fetchLike });

  await assert.rejects(
    () => simulateAndSubmit({ rpc: client, xdr: BASE_XDR, sign: NOOP_SIGN }),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'SUBMISSION_NETWORK_ERROR');
      assert.equal(err.retryable, true);
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// simulateAndSubmit – on-chain rejection
// ---------------------------------------------------------------------------

test('simulateAndSubmit maps FAILED getTransaction status to ONCHAIN_REJECTED', async () => {
  const getFailure: GetTransactionResult = { status: 'FAILED', resultXdr: 'failedXdr==' };
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([
      rpcOk(SIM_OK),
      rpcOk(SEND_PENDING),
      rpcOk(getFailure),
    ]),
  });

  await assert.rejects(
    () => simulateAndSubmit({ rpc: client, xdr: BASE_XDR, sign: NOOP_SIGN }),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'ONCHAIN_REJECTED');
      assert.equal(err.retryable, false);
      assert.equal((err as SdkCoreError).txStatus, 'FAILED');
      assert.equal((err as SdkCoreError).resultXdr, 'failedXdr==');
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// simulateAndSubmit – result wait timeout
// ---------------------------------------------------------------------------

test('simulateAndSubmit maps poll timeout to RESULT_WAIT_TIMEOUT', async () => {
  const getNotFound: GetTransactionResult = { status: 'NOT_FOUND' };
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([
      rpcOk(SIM_OK),
      rpcOk(SEND_PENDING),
      // All subsequent getTransaction calls return NOT_FOUND
      rpcOk(getNotFound),
      rpcOk(getNotFound),
    ]),
  });

  await assert.rejects(
    () =>
      simulateAndSubmit({
        rpc: client,
        xdr: BASE_XDR,
        sign: NOOP_SIGN,
        pollTimeoutMs: 50, // very short timeout for the test
        pollIntervalMs: 10,
      }),
    (err: unknown) => {
      assert.ok(isSdkCoreError(err));
      assert.equal(err.code, 'RESULT_WAIT_TIMEOUT');
      assert.equal(err.retryable, true);
      assert.equal((err as SdkCoreError).txHash, 'abc123');
      return true;
    },
  );
});

// ---------------------------------------------------------------------------
// simulateAndSubmit – happy path
// ---------------------------------------------------------------------------

test('simulateAndSubmit returns SubmitResult on success', async () => {
  const client = new SorobanRpcClient({
    url: 'https://rpc.example',
    fetch: mockFetch([
      rpcOk(SIM_OK),
      rpcOk(SEND_PENDING),
      rpcOk(GET_SUCCESS),
    ]),
  });

  const result = await simulateAndSubmit({ rpc: client, xdr: BASE_XDR, sign: NOOP_SIGN });

  assert.equal(result.txHash, 'abc123');
  assert.equal(result.status, 'SUCCESS');
  assert.equal(result.resultXdr, 'resultXdrBase64==');
  assert.equal(result.ledger, 42);
});

// ---------------------------------------------------------------------------
// All four variants appear in the union – exhaustiveness check via switch
// ---------------------------------------------------------------------------

test('all four SdkError kinds are handled exhaustively in a switch', () => {
  const errors: SdkCoreError[] = [
    SdkCoreError.simulationFailed({ message: 'a' }),
    SdkCoreError.networkError({ message: 'b' }),
    SdkCoreError.rpcError({ message: 'c' }),
    SdkCoreError.onchainRejected({ message: 'd', txStatus: 'FAILED' }),
    SdkCoreError.resultWaitTimeout({ message: 'e' }),
  ];

  const seen = new Set<string>();
  for (const err of errors) {
    switch (err.kind) {
      case 'SimulationError':
        seen.add('SimulationError');
        break;
      case 'SubmissionError':
        seen.add('SubmissionError');
        break;
      case 'OnchainRejectedError':
        seen.add('OnchainRejectedError');
        break;
      case 'TimeoutError':
        seen.add('TimeoutError');
        break;
      default: {
        // TypeScript exhaustiveness: the `never` cast below ensures the
        // compiler catches unhandled variants at build time.
        const _exhaustive: never = err.kind;
        assert.fail(`Unhandled kind: ${String(_exhaustive)}`);
      }
    }
  }

  assert.ok(seen.has('SimulationError'), 'SimulationError not seen');
  assert.ok(seen.has('SubmissionError'), 'SubmissionError not seen');
  assert.ok(seen.has('OnchainRejectedError'), 'OnchainRejectedError not seen');
  assert.ok(seen.has('TimeoutError'), 'TimeoutError not seen');
});
