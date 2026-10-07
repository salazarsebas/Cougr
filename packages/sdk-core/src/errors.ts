/**
 * Versioned discriminated-union error taxonomy for the sdk-core simulate-and-submit path.
 *
 * Every variant carries:
 *   - `code`      – a stable string identifier.  Renaming a code is a breaking change.
 *   - `retryable` – whether the caller MAY safely retry the same transaction unchanged.
 *   - `message`   – a human-readable description of the failure.
 *   - `cause`     – the original underlying error or raw RPC body, when available.
 *
 * ## Taxonomy version
 * TAXONOMY_VERSION = "1"
 *
 * ## Codes at a glance
 *
 * | Code                        | Variant             | retryable |
 * |-----------------------------|---------------------|-----------|
 * | SIMULATION_FAILED           | SimulationError     | false     |
 * | SUBMISSION_NETWORK_ERROR    | SubmissionError     | true      |
 * | SUBMISSION_RPC_ERROR        | SubmissionError     | false     |
 * | ONCHAIN_REJECTED            | OnchainRejectedError| false     |
 * | RESULT_WAIT_TIMEOUT         | TimeoutError        | true      |
 *
 * See README.md for the full catalogue with retry semantics.
 */

/** Taxonomy schema version.  Increment whenever a code is added, removed, or renamed. */
export const TAXONOMY_VERSION = '1' as const;

// ---------------------------------------------------------------------------
// Base interface
// ---------------------------------------------------------------------------

/**
 * Common fields shared by every error variant.
 * Do not construct this directly; use the concrete subtype or {@link SdkError}.
 */
export interface SdkErrorBase {
  /** Discriminant tag – matches the class name and the union literal. */
  readonly kind: SdkErrorKind;
  /** Stable string code. A rename is a breaking change. */
  readonly code: SdkErrorCode;
  /** `true` if retrying the same transaction (unchanged) is safe. */
  readonly retryable: boolean;
  /** Human-readable explanation. */
  readonly message: string;
  /** The original thrown value or raw RPC body, when available. */
  readonly cause?: unknown;
}

// ---------------------------------------------------------------------------
// Kind and code enumerations
// ---------------------------------------------------------------------------

/** Discriminant tags for the union. */
export type SdkErrorKind =
  | 'SimulationError'
  | 'SubmissionError'
  | 'OnchainRejectedError'
  | 'TimeoutError';

/** Stable string codes. Renaming any member is a breaking change. */
export type SdkErrorCode =
  | 'SIMULATION_FAILED'
  | 'SUBMISSION_NETWORK_ERROR'
  | 'SUBMISSION_RPC_ERROR'
  | 'ONCHAIN_REJECTED'
  | 'RESULT_WAIT_TIMEOUT';

// ---------------------------------------------------------------------------
// Variant 1 – SimulationError
// ---------------------------------------------------------------------------

/**
 * The Soroban simulation step rejected the transaction before it was sent.
 *
 * - **code** `SIMULATION_FAILED`
 * - **retryable** `false` — simulation failures are deterministic; retrying
 *   without changing the transaction will produce the same result.
 *
 * `contractMessage` captures the short reason string that Soroban contracts
 * may surface (e.g. the turn-based template's `"not your turn"` reason) so
 * callers can branch on it without parsing `message`.
 */
export interface SimulationError extends SdkErrorBase {
  readonly kind: 'SimulationError';
  readonly code: 'SIMULATION_FAILED';
  readonly retryable: false;
  /** The raw error string returned by the RPC simulation response, if any. */
  readonly simulationError?: string;
  /**
   * A structured reason from the contract itself, extracted from the
   * simulation error when the contract returns `{ success: false, reason }`.
   */
  readonly contractMessage?: string;
}

// ---------------------------------------------------------------------------
// Variant 2 – SubmissionError
// ---------------------------------------------------------------------------

/**
 * The transaction was built and signed but could not be submitted.
 *
 * Two sub-codes distinguish retryable from non-retryable submission failures:
 *
 * - `SUBMISSION_NETWORK_ERROR` – a transport-level failure (connection reset,
 *   DNS, timeout before the RPC even responded).  **retryable = true**.
 * - `SUBMISSION_RPC_ERROR` – the RPC endpoint responded with a JSON-RPC
 *   error or an unexpected HTTP status.  **retryable = false** (the RPC
 *   rejected the payload; changing nothing and retrying is unlikely to help).
 */
export interface SubmissionError extends SdkErrorBase {
  readonly kind: 'SubmissionError';
  readonly code: 'SUBMISSION_NETWORK_ERROR' | 'SUBMISSION_RPC_ERROR';
  /** HTTP status code, when the failure was an HTTP response. */
  readonly httpStatus?: number;
  /** JSON-RPC error code, when the failure was a JSON-RPC error body. */
  readonly rpcCode?: number;
}

// ---------------------------------------------------------------------------
// Variant 3 – OnchainRejectedError
// ---------------------------------------------------------------------------

/**
 * The transaction reached the network and was included in a ledger, but the
 * ledger status is `FAILED` or equivalent.
 *
 * - **code** `ONCHAIN_REJECTED`
 * - **retryable** `false` — the ledger has already decided; the same
 *   transaction will not be accepted again.
 *
 * `resultXdr` preserves the raw XDR result so callers can decode the exact
 * contract error code or operation result without re-querying the RPC.
 */
export interface OnchainRejectedError extends SdkErrorBase {
  readonly kind: 'OnchainRejectedError';
  readonly code: 'ONCHAIN_REJECTED';
  readonly retryable: false;
  /** The final transaction status as reported by the RPC (e.g. `"FAILED"`). */
  readonly txStatus: string;
  /** Raw base-64 XDR result envelope, when available. */
  readonly resultXdr?: string;
}

// ---------------------------------------------------------------------------
// Variant 4 – TimeoutError
// ---------------------------------------------------------------------------

/**
 * The transaction was submitted successfully but a result was not observed
 * within the configured deadline.
 *
 * - **code** `RESULT_WAIT_TIMEOUT`
 * - **retryable** `true` — the transaction may still be in-flight; the
 *   caller should re-query by hash before attempting a fresh submission.
 *
 * `txHash` lets the caller poll the RPC directly to determine whether the
 * transaction eventually landed.
 */
export interface TimeoutError extends SdkErrorBase {
  readonly kind: 'TimeoutError';
  readonly code: 'RESULT_WAIT_TIMEOUT';
  readonly retryable: true;
  /** Transaction hash of the submitted transaction, if available. */
  readonly txHash?: string;
  /** Deadline that elapsed, in milliseconds. */
  readonly deadlineMs?: number;
}

// ---------------------------------------------------------------------------
// Discriminated union
// ---------------------------------------------------------------------------

/**
 * The full simulate-and-submit error union.
 *
 * Narrow it with a `switch` on `kind` or a `code` equality check:
 *
 * ```ts
 * if (err instanceof SdkCoreError) {
 *   switch (err.kind) {
 *     case 'SimulationError':   // err: SimulationError
 *     case 'SubmissionError':   // err: SubmissionError
 *     case 'OnchainRejectedError': // err: OnchainRejectedError
 *     case 'TimeoutError':      // err: TimeoutError
 *   }
 * }
 * ```
 */
export type SdkError =
  | SimulationError
  | SubmissionError
  | OnchainRejectedError
  | TimeoutError;

// ---------------------------------------------------------------------------
// Concrete Error class
// ---------------------------------------------------------------------------

/**
 * `SdkCoreError` is an `Error` subclass that also implements every field of
 * the {@link SdkError} union, so it can be thrown, caught with `instanceof`,
 * and narrowed by `kind`/`code` in the same expression.
 *
 * Use the static factory helpers rather than the constructor directly.
 */
export class SdkCoreError extends Error implements SdkErrorBase {
  readonly kind: SdkErrorKind;
  readonly code: SdkErrorCode;
  readonly retryable: boolean;
  readonly cause?: unknown;

  // Variant-specific optional fields — only populated for the matching variant.
  readonly simulationError?: string;
  readonly contractMessage?: string;
  readonly httpStatus?: number;
  readonly rpcCode?: number;
  readonly txStatus?: string;
  readonly resultXdr?: string;
  readonly txHash?: string;
  readonly deadlineMs?: number;

  private constructor(fields: SdkError) {
    super(fields.message);
    this.name = 'SdkCoreError';
    this.kind = fields.kind;
    this.code = fields.code;
    this.retryable = fields.retryable;
    this.cause = fields.cause;

    // Copy variant-specific fields when present.
    if ('simulationError' in fields) this.simulationError = fields.simulationError;
    if ('contractMessage' in fields) this.contractMessage = fields.contractMessage;
    if ('httpStatus' in fields) this.httpStatus = fields.httpStatus;
    if ('rpcCode' in fields) this.rpcCode = fields.rpcCode;
    if ('txStatus' in fields) this.txStatus = fields.txStatus;
    if ('resultXdr' in fields) this.resultXdr = fields.resultXdr;
    if ('txHash' in fields) this.txHash = fields.txHash;
    if ('deadlineMs' in fields) this.deadlineMs = fields.deadlineMs;
  }

  // -------------------------------------------------------------------------
  // Static factories
  // -------------------------------------------------------------------------

  /**
   * Create a {@link SimulationError} (`SIMULATION_FAILED`).
   *
   * @param opts.simulationError - Raw error string from the RPC simulation response.
   * @param opts.contractMessage - Structured reason from the contract, if parseable.
   * @param opts.cause           - Original thrown value.
   */
  static simulationFailed(opts: {
    message: string;
    simulationError?: string;
    contractMessage?: string;
    cause?: unknown;
  }): SdkCoreError {
    return new SdkCoreError({
      kind: 'SimulationError',
      code: 'SIMULATION_FAILED',
      retryable: false,
      message: opts.message,
      simulationError: opts.simulationError,
      contractMessage: opts.contractMessage,
      cause: opts.cause,
    });
  }

  /**
   * Create a {@link SubmissionError} for a transport-level failure
   * (`SUBMISSION_NETWORK_ERROR`, **retryable**).
   */
  static networkError(opts: { message: string; cause?: unknown }): SdkCoreError {
    return new SdkCoreError({
      kind: 'SubmissionError',
      code: 'SUBMISSION_NETWORK_ERROR',
      retryable: true,
      message: opts.message,
      cause: opts.cause,
    });
  }

  /**
   * Create a {@link SubmissionError} for an RPC-level rejection
   * (`SUBMISSION_RPC_ERROR`, **not retryable**).
   *
   * @param opts.httpStatus - HTTP status code when available.
   * @param opts.rpcCode    - JSON-RPC error code when available.
   */
  static rpcError(opts: {
    message: string;
    httpStatus?: number;
    rpcCode?: number;
    cause?: unknown;
  }): SdkCoreError {
    return new SdkCoreError({
      kind: 'SubmissionError',
      code: 'SUBMISSION_RPC_ERROR',
      retryable: false,
      message: opts.message,
      httpStatus: opts.httpStatus,
      rpcCode: opts.rpcCode,
      cause: opts.cause,
    });
  }

  /**
   * Create an {@link OnchainRejectedError} (`ONCHAIN_REJECTED`).
   *
   * @param opts.txStatus   - Status string from the RPC (e.g. `"FAILED"`).
   * @param opts.resultXdr  - Raw base-64 XDR result envelope, when available.
   */
  static onchainRejected(opts: {
    message: string;
    txStatus: string;
    resultXdr?: string;
    cause?: unknown;
  }): SdkCoreError {
    return new SdkCoreError({
      kind: 'OnchainRejectedError',
      code: 'ONCHAIN_REJECTED',
      retryable: false,
      message: opts.message,
      txStatus: opts.txStatus,
      resultXdr: opts.resultXdr,
      cause: opts.cause,
    });
  }

  /**
   * Create a {@link TimeoutError} (`RESULT_WAIT_TIMEOUT`, **retryable**).
   *
   * @param opts.txHash     - Hash of the submitted transaction.
   * @param opts.deadlineMs - Elapsed deadline in milliseconds.
   */
  static resultWaitTimeout(opts: {
    message: string;
    txHash?: string;
    deadlineMs?: number;
    cause?: unknown;
  }): SdkCoreError {
    return new SdkCoreError({
      kind: 'TimeoutError',
      code: 'RESULT_WAIT_TIMEOUT',
      retryable: true,
      message: opts.message,
      txHash: opts.txHash,
      deadlineMs: opts.deadlineMs,
      cause: opts.cause,
    });
  }

  // -------------------------------------------------------------------------
  // Type guard helpers
  // -------------------------------------------------------------------------

  /** Narrows to {@link SimulationError}. */
  isSimulationError(): this is this & SimulationError {
    return this.kind === 'SimulationError';
  }

  /** Narrows to {@link SubmissionError}. */
  isSubmissionError(): this is this & SubmissionError {
    return this.kind === 'SubmissionError';
  }

  /** Narrows to {@link OnchainRejectedError}. */
  isOnchainRejected(): this is this & OnchainRejectedError {
    return this.kind === 'OnchainRejectedError';
  }

  /** Narrows to {@link TimeoutError}. */
  isTimeout(): this is this & TimeoutError {
    return this.kind === 'TimeoutError';
  }
}

// ---------------------------------------------------------------------------
// Standalone type-guard functions (for use without instanceof)
// ---------------------------------------------------------------------------

/** Returns `true` if `err` is an `SdkCoreError`. */
export function isSdkCoreError(err: unknown): err is SdkCoreError {
  return err instanceof SdkCoreError;
}

/** Returns `true` if `err` is an `SdkCoreError` with `kind === 'SimulationError'`. */
export function isSimulationError(err: unknown): err is SdkCoreError & SimulationError {
  return err instanceof SdkCoreError && err.kind === 'SimulationError';
}

/** Returns `true` if `err` is an `SdkCoreError` with `kind === 'SubmissionError'`. */
export function isSubmissionError(err: unknown): err is SdkCoreError & SubmissionError {
  return err instanceof SdkCoreError && err.kind === 'SubmissionError';
}

/** Returns `true` if `err` is an `SdkCoreError` with `kind === 'OnchainRejectedError'`. */
export function isOnchainRejected(err: unknown): err is SdkCoreError & OnchainRejectedError {
  return err instanceof SdkCoreError && err.kind === 'OnchainRejectedError';
}

/** Returns `true` if `err` is an `SdkCoreError` with `kind === 'TimeoutError'`. */
export function isTimeoutError(err: unknown): err is SdkCoreError & TimeoutError {
  return err instanceof SdkCoreError && err.kind === 'TimeoutError';
}
