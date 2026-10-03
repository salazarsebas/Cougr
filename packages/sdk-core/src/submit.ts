/**
 * Soroban RPC types and a minimal simulate-and-submit client for Cougr contracts.
 *
 * This module provides:
 *   - {@link SorobanRpcClient} – a thin JSON-RPC client used internally and in tests.
 *   - {@link simulateAndSubmit} – runs the full simulate → sign → submit → poll cycle,
 *     mapping every failure mode to the appropriate {@link SdkCoreError} variant.
 *
 * The client is intentionally minimal: it does not add retry logic (that is
 * the caller's responsibility, guided by `SdkCoreError.retryable`) and it does
 * not sign transactions (inject a signer via {@link SubmitOptions.sign}).
 */

import { SdkCoreError } from './errors.ts';

// ---------------------------------------------------------------------------
// Fetch abstraction (mirrors sdk-events pattern)
// ---------------------------------------------------------------------------

/** Minimal structural `fetch` contract so the SDK works in any runtime. */
export interface FetchLike {
  (
    input: string,
    init: { method: string; headers: Record<string, string>; body: string },
  ): Promise<{
    ok: boolean;
    status: number;
    json(): Promise<unknown>;
    text(): Promise<string>;
  }>;
}

// ---------------------------------------------------------------------------
// Raw RPC shapes (typed minimally – only fields sdk-core needs)
// ---------------------------------------------------------------------------

export interface SimulateTransactionResult {
  /** Present when simulation succeeds; contains fee and footprint data. */
  results?: Array<{ xdr: string }>;
  /** Present when simulation fails; contains the error string. */
  error?: string;
  /** Pre-auth footprint XDR. */
  transactionData?: string;
  /** Minimum resource fee in stroops. */
  minResourceFee?: string;
  /** Raw events emitted during simulation. */
  events?: unknown[];
}

export interface SendTransactionResult {
  /** Transaction hash. */
  hash: string;
  /**
   * Immediate submission status.
   * `PENDING` means the network accepted the envelope; poll to get the final
   * ledger status.  Anything else is a rejection.
   */
  status: string;
  /** RPC error details, present when `status` is not `PENDING`. */
  errorResultXdr?: string;
}

export interface GetTransactionResult {
  /**
   * `SUCCESS`, `FAILED`, or `NOT_FOUND`.
   * `NOT_FOUND` means the transaction has not yet been included in a ledger.
   */
  status: 'SUCCESS' | 'FAILED' | 'NOT_FOUND' | string;
  /** Base-64 XDR result envelope, present on `SUCCESS` or `FAILED`. */
  resultXdr?: string;
  /** Ledger in which the transaction was included. */
  ledger?: number;
}

// ---------------------------------------------------------------------------
// RPC client
// ---------------------------------------------------------------------------

export interface SorobanRpcClientOptions {
  /** Soroban RPC endpoint, e.g. `https://soroban-testnet.stellar.org`. */
  url: string;
  /** Inject a custom fetch for tests or custom auth. Defaults to `globalThis.fetch`. */
  fetch?: FetchLike;
  headers?: Record<string, string>;
}

interface JsonRpcResponse {
  result?: unknown;
  error?: { code?: number; message?: string; data?: unknown };
}

/** Minimal Soroban JSON-RPC client used by {@link simulateAndSubmit}. */
export class SorobanRpcClient {
  readonly url: string;
  private readonly _fetch: FetchLike;
  private readonly _headers: Record<string, string>;

  constructor(options: SorobanRpcClientOptions) {
    this.url = options.url;
    const f = options.fetch ?? (globalThis.fetch as unknown as FetchLike | undefined);
    if (f === undefined) {
      throw new Error('No fetch implementation available; pass options.fetch');
    }
    this._fetch = f;
    this._headers = options.headers ?? {};
  }

  /** Send a raw JSON-RPC request and return the `result` field. */
  async request<T>(method: string, params: unknown): Promise<T> {
    let response: { ok: boolean; status: number; json(): Promise<unknown>; text(): Promise<string> };
    try {
      response = await this._fetch(this.url, {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...this._headers },
        body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params: [params] }),
      });
    } catch (cause) {
      throw SdkCoreError.networkError({
        message: `Network error calling ${method}: ${String(cause)}`,
        cause,
      });
    }

    if (!response.ok) {
      const body = await response.text().catch(() => '');
      throw SdkCoreError.rpcError({
        message: `HTTP ${response.status} calling ${method}: ${body}`,
        httpStatus: response.status,
        cause: body,
      });
    }

    let body: JsonRpcResponse;
    try {
      body = (await response.json()) as JsonRpcResponse;
    } catch (cause) {
      throw SdkCoreError.rpcError({
        message: `Malformed JSON response from ${method}`,
        httpStatus: response.status,
        cause,
      });
    }

    if (body.error !== undefined) {
      throw SdkCoreError.rpcError({
        message: `RPC error ${body.error.code ?? ''} calling ${method}: ${body.error.message ?? 'unknown'}`,
        rpcCode: body.error.code,
        cause: body.error,
      });
    }

    return body.result as T;
  }

  async simulateTransaction(xdr: string): Promise<SimulateTransactionResult> {
    return this.request<SimulateTransactionResult>('simulateTransaction', { transaction: xdr });
  }

  async sendTransaction(xdr: string): Promise<SendTransactionResult> {
    return this.request<SendTransactionResult>('sendTransaction', { transaction: xdr });
  }

  async getTransaction(hash: string): Promise<GetTransactionResult> {
    return this.request<GetTransactionResult>('getTransaction', { hash });
  }
}

// ---------------------------------------------------------------------------
// simulateAndSubmit
// ---------------------------------------------------------------------------

/**
 * Options for {@link simulateAndSubmit}.
 */
export interface SubmitOptions {
  /** Soroban RPC client. */
  rpc: SorobanRpcClient;
  /**
   * Base-64 XDR transaction envelope to simulate and submit.
   * The envelope should be unsigned; {@link sign} will be called after
   * simulation augments the footprint and fee.
   */
  xdr: string;
  /**
   * Async function that receives the simulation-augmented XDR and returns a
   * signed XDR ready for submission.
   */
  sign: (augmentedXdr: string) => Promise<string>;
  /**
   * How long to poll for a ledger result, in milliseconds.
   * @default 30_000
   */
  pollTimeoutMs?: number;
  /**
   * Interval between poll attempts, in milliseconds.
   * @default 2_000
   */
  pollIntervalMs?: number;
}

/** Successful result from {@link simulateAndSubmit}. */
export interface SubmitResult {
  /** Transaction hash. */
  txHash: string;
  /** Final ledger status (`"SUCCESS"`). */
  status: string;
  /** Raw base-64 XDR result envelope. */
  resultXdr: string;
  /** Ledger in which the transaction was included. */
  ledger: number;
}

/**
 * Run the full simulate → augment → sign → submit → poll lifecycle.
 *
 * Throws a {@link SdkCoreError} on any failure, with `code` and `retryable`
 * set according to the taxonomy in `errors.ts`.
 *
 * ```ts
 * try {
 *   const result = await simulateAndSubmit({ rpc, xdr, sign });
 *   console.log('tx:', result.txHash);
 * } catch (err) {
 *   if (isSdkCoreError(err)) {
 *     console.error(err.code, 'retryable:', err.retryable);
 *   }
 * }
 * ```
 */
export async function simulateAndSubmit(options: SubmitOptions): Promise<SubmitResult> {
  const {
    rpc,
    xdr,
    sign,
    pollTimeoutMs = 30_000,
    pollIntervalMs = 2_000,
  } = options;

  // ------------------------------------------------------------------
  // Step 1 – Simulate
  // ------------------------------------------------------------------
  const sim = await rpc.simulateTransaction(xdr);
  // SorobanRpcClient.request already maps network/RPC failures to SdkCoreError.
  // We only need to handle the case where simulation itself returns an error.

  if (sim.error !== undefined) {
    // Attempt to extract a structured contract reason.  Contracts that return
    // `{ success: false, reason: "..." }` may surface the reason inside the
    // simulation error string.
    const contractMessage = extractContractReason(sim.error);
    throw SdkCoreError.simulationFailed({
      message: `Simulation failed: ${sim.error}`,
      simulationError: sim.error,
      contractMessage,
      cause: sim,
    });
  }

  // Augment the transaction XDR with simulation footprint + fee.
  const augmentedXdr = augmentWithSimulation(xdr, sim);

  // ------------------------------------------------------------------
  // Step 2 – Sign
  // ------------------------------------------------------------------
  let signedXdr: string;
  try {
    signedXdr = await sign(augmentedXdr);
  } catch (cause) {
    // Signing errors are not SdkCoreErrors – re-throw as-is so the caller
    // can distinguish them.
    throw cause;
  }

  // ------------------------------------------------------------------
  // Step 3 – Submit
  // ------------------------------------------------------------------
  const submission = await rpc.sendTransaction(signedXdr);
  // Again, network/RPC-level failures are already mapped by the client.

  if (submission.status !== 'PENDING') {
    throw SdkCoreError.rpcError({
      message: `sendTransaction returned unexpected status "${submission.status}"`,
      cause: submission,
    });
  }

  const txHash = submission.hash;

  // ------------------------------------------------------------------
  // Step 4 – Poll for result
  // ------------------------------------------------------------------
  const deadline = Date.now() + pollTimeoutMs;

  while (Date.now() < deadline) {
    const result = await rpc.getTransaction(txHash);

    if (result.status === 'SUCCESS') {
      return {
        txHash,
        status: result.status,
        resultXdr: result.resultXdr ?? '',
        ledger: result.ledger ?? 0,
      };
    }

    if (result.status === 'FAILED') {
      throw SdkCoreError.onchainRejected({
        message: `Transaction ${txHash} was rejected on-chain with status FAILED`,
        txStatus: result.status,
        resultXdr: result.resultXdr,
        cause: result,
      });
    }

    // status === 'NOT_FOUND' or other transient value: keep polling.
    await sleep(pollIntervalMs);
  }

  throw SdkCoreError.resultWaitTimeout({
    message: `Timed out waiting for transaction ${txHash} after ${pollTimeoutMs}ms`,
    txHash,
    deadlineMs: pollTimeoutMs,
  });
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Augment a base transaction XDR with footprint and fee data from simulation.
 *
 * In a full implementation this would use the Stellar SDK's
 * `assembleTransaction` helper.  Here we return the original XDR unchanged
 * so that sdk-core has no Stellar SDK dependency; real callers inject
 * augmented XDR via their own wrapper or by calling `assembleTransaction`
 * before passing to `sign`.
 *
 * @internal
 */
export function augmentWithSimulation(xdr: string, _sim: SimulateTransactionResult): string {
  // Real implementations would call:
  //   import { assembleTransaction } from '@stellar/stellar-sdk/minimal';
  //   return assembleTransaction(xdr, _sim).toXDR('base64');
  // We leave this as a pass-through to avoid a hard SDK dependency in core.
  return xdr;
}

/**
 * Try to extract a contract-level reason string from a simulation error.
 * Contracts returning `{ success: false, reason: "..." }` embed the reason
 * somewhere in the diagnostic; this heuristic extracts it when present.
 *
 * @internal
 */
export function extractContractReason(errorString: string): string | undefined {
  // Look for patterns like `reason: "not your turn"` or `"reason":"..."`.
  const match =
    /["']?reason["']?\s*:\s*["']([^"']+)["']/.exec(errorString) ??
    /\breason\s*=\s*([^\s,}]+)/.exec(errorString);
  return match?.[1];
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
