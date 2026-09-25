/**
 * TypeScript session client for Cougr.
 *
 * This package mirrors the on-chain session model built by
 * `cougr_core::accounts::SessionBuilder` and enforced by `AccountKernel` /
 * `SessionPolicy`:
 *
 * - `SessionScope` and `SessionKey` keep the Rust field meaning, byte order, and
 *   expiry units (ledger seconds).
 * - Expiry uses the same boundary as the contract: `now >= expiresAt` is expired.
 * - Replay protection reuses the same per-session `nextNonce` counter, verified
 *   before submit so a reused nonce never reaches the network.
 * - `deriveSessionKeyId` reproduces the 32-byte id the contract derives in
 *   `src/accounts/contract.rs`, so a client can predict the key it is creating.
 *
 * There is no dependency on the RPC stack. Where a key must be created or a
 * transaction submitted, the caller passes a `SessionKeyProvider` (see README).
 */

/** Scope of a session key, mirroring `cougr_core::accounts::SessionScope`. */
export interface SessionScope {
  /** Action symbols the session may authorize, in builder order. */
  readonly allowedActions: string[];
  /** Maximum number of operations before the session is exhausted. */
  readonly maxOperations: number;
  /** Ledger timestamp (seconds) at which the session expires. */
  readonly expiresAt: bigint;
}

/** A session key with its scope and usage tracking, mirroring `SessionKey`. */
export interface SessionKey {
  /** 32-byte session key id (`BytesN<32>` on chain). */
  readonly keyId: Uint8Array;
  readonly scope: SessionScope;
  /** Ledger timestamp (seconds) when the session was created. */
  readonly createdAt: bigint;
  /** Operations already consumed. */
  readonly operationsUsed: number;
  /** Nonce the next authorized action must carry. */
  readonly nextNonce: bigint;
}

/** Ledger view the rules run against, mirroring `env.ledger()`. */
export interface LedgerClock {
  timestamp(): bigint;
  sequence(): number;
}

/** Everything the package needs from a chain client, kept behind one interface. */
export interface SessionKeyProvider {
  createSession(scope: SessionScope): SessionKey | Promise<SessionKey>;
}

/** Contract error codes from `cougr_core::accounts::AccountError`. */
export const SESSION_ERROR = {
  SessionExpired: 21,
  InvalidSignature: 22,
  SessionLimitReached: 24,
  InvalidScope: 25,
  NonceMismatch: 37,
  ActionNotAllowed: 38,
  SessionBudgetExceeded: 39,
  SessionRevoked: 43,
} as const;

export type SessionErrorName = keyof typeof SESSION_ERROR;

/** Error carrying the same numeric code the contract returns. */
export class SessionError extends Error {
  readonly code: number;
  readonly sessionError: SessionErrorName;

  constructor(name: SessionErrorName) {
    super(`${name} (contract error ${SESSION_ERROR[name]})`);
    this.name = 'SessionError';
    this.sessionError = name;
    this.code = SESSION_ERROR[name];
  }
}

/** Outcome of `authorizeSession`, mirroring the contract accept/reject split. */
export type AuthorizationResult =
  | {
      readonly ok: true;
      readonly key: SessionKey;
      readonly remainingOperations: number;
    }
  | { readonly ok: false; readonly error: SessionError };

const ACTION_PATTERN = /^[a-z0-9_]{1,32}$/;

function assertAction(action: string): void {
  if (!ACTION_PATTERN.test(action)) {
    throw new SessionError('InvalidScope');
  }
}

function assertScope(scope: SessionScope): void {
  if (!Number.isInteger(scope.maxOperations) || scope.maxOperations < 0) {
    throw new SessionError('InvalidScope');
  }
  for (const action of scope.allowedActions) {
    assertAction(action);
  }
}

/** Fluent builder mirroring `SessionBuilder`, including `expires_in`. */
export class SessionBuilder {
  readonly #clock: LedgerClock;
  #allowedActions: string[] = [];
  #maxOperations = 0;
  #expiresAt = 0n;

  constructor(clock: LedgerClock) {
    this.#clock = clock;
  }

  allowAction(action: string): this {
    assertAction(action);
    this.#allowedActions.push(action);
    return this;
  }

  maxOperations(count: number): this {
    if (!Number.isInteger(count) || count < 0) {
      throw new SessionError('InvalidScope');
    }
    this.#maxOperations = count;
    return this;
  }

  expiresAt(timestamp: bigint): this {
    this.#expiresAt = timestamp;
    return this;
  }

  expiresIn(seconds: bigint): this {
    this.#expiresAt = this.#clock.timestamp() + seconds;
    return this;
  }

  buildScope(): SessionScope {
    const scope: SessionScope = {
      allowedActions: [...this.#allowedActions],
      maxOperations: this.#maxOperations,
      expiresAt: this.#expiresAt,
    };
    assertScope(scope);
    return scope;
  }

  async create(provider: SessionKeyProvider): Promise<SessionKey> {
    return provider.createSession(this.buildScope());
  }
}

/** True when the contract would treat the session as expired (`now >= expiresAt`). */
export function isExpired(key: SessionKey, clock: LedgerClock): boolean {
  return clock.timestamp() >= key.scope.expiresAt;
}

/** Mirrors `ContractAccount::validate_session`. */
export function validateSession(key: SessionKey, clock: LedgerClock): boolean {
  if (isExpired(key, clock)) {
    return false;
  }
  return key.operationsUsed < key.scope.maxOperations;
}

/**
 * Mirrors `SessionPolicy::evaluate` followed by
 * `SessionStorage::consume_authorized_session`.
 *
 * Checks run in the contract's order: expiry, budget, replay nonce, then the
 * allowed-action list. On success the returned key has advanced by one
 * operation and one nonce, exactly like the stored key after the call.
 */
export function authorizeSession(
  key: SessionKey,
  action: string,
  nonce: bigint,
  clock: LedgerClock,
): AuthorizationResult {
  if (isExpired(key, clock)) {
    return { ok: false, error: new SessionError('SessionExpired') };
  }
  if (key.operationsUsed >= key.scope.maxOperations) {
    return { ok: false, error: new SessionError('SessionBudgetExceeded') };
  }
  if (key.nextNonce !== nonce) {
    return { ok: false, error: new SessionError('NonceMismatch') };
  }
  if (!key.scope.allowedActions.includes(action)) {
    return { ok: false, error: new SessionError('ActionNotAllowed') };
  }

  const next: SessionKey = {
    ...key,
    operationsUsed: key.operationsUsed + 1,
    nextNonce: key.nextNonce + 1n,
  };
  return {
    ok: true,
    key: next,
    remainingOperations: next.scope.maxOperations - next.operationsUsed,
  };
}

/**
 * Stateful client that tracks one key and rejects stale expiry or nonces before
 * a transaction is built.
 */
export class SessionClient {
  readonly #clock: LedgerClock;
  #key: SessionKey;

  constructor(key: SessionKey, clock: LedgerClock) {
    this.#key = key;
    this.#clock = clock;
  }

  get key(): SessionKey {
    return this.#key;
  }

  validate(): boolean {
    return validateSession(this.#key, this.#clock);
  }

  /** Reject a consumed or future nonce locally, before submit. */
  assertFreshNonce(nonce: bigint): void {
    if (nonce !== this.#key.nextNonce) {
      throw new SessionError('NonceMismatch');
    }
  }

  authorize(action: string, nonce: bigint): AuthorizationResult {
    const result = authorizeSession(this.#key, action, nonce, this.#clock);
    if (result.ok) {
      this.#key = result.key;
    }
    return result;
  }
}

/**
 * Derive the 32-byte session key id the contract stores.
 *
 * Layout, all big-endian, matching `session_key_id` in
 * `src/accounts/contract.rs`:
 *
 * ```text
 * timestamp (u64, 8) | sequence (u32, 4) | existingSessions (u32, 4)
 * | allowedActions.length (u32, 4) | maxOperations (u32, 4) | expiresAt (u64, 8)
 * ```
 */
export function deriveSessionKeyId(input: {
  timestamp: bigint;
  sequence: number;
  existingSessions: number;
  scope: SessionScope;
}): Uint8Array {
  const { timestamp, sequence, existingSessions, scope } = input;
  assertScope(scope);

  const bytes = new Uint8Array(32);
  const view = new DataView(bytes.buffer);
  view.setBigUint64(0, BigInt.asUintN(64, timestamp), false);
  view.setUint32(8, sequence, false);
  view.setUint32(12, existingSessions, false);
  view.setUint32(16, scope.allowedActions.length, false);
  view.setUint32(20, scope.maxOperations, false);
  view.setBigUint64(24, BigInt.asUintN(64, scope.expiresAt), false);
  return bytes;
}

/** Lowercase hex, the encoding the committed cross-language vectors use. */
export function toHex(bytes: Uint8Array): string {
  let out = '';
  for (const byte of bytes) {
    out += byte.toString(16).padStart(2, '0');
  }
  return out;
}
