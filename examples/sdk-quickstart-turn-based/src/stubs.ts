/**
 * Local stubs for sdk-core (#350) and sdk-session (#351).
 *
 * These replace the stand-in packages that were removed in PR #367.  Once
 * #350 (cougr-sdk-core) and #351 (cougr-sdk-session) are merged to main,
 * delete this file and import from those packages directly.
 *
 * Only the surface used by quickstart.ts and quickstart.test.ts is stubbed:
 *   sdk-core:    TurnBasedClient, decodeMoveResult, FIXTURE_PLAYER_X/O,
 *                FIXTURE_INITIAL_STATE, FIXTURE_AFTER_MOVE_STATE
 *   sdk-session: SessionBuilder, buildSessionAuth, FIXTURE_SESSION_SEED
 */

import type { GameState } from './game-state.ts';

// ── sdk-core types ─────────────────────────────────────────────────────────

export interface MoveResult {
  success: boolean;
  game_state: GameState;
  message: string;  // "ok" | "gameover" | "bounds" | "occupied" | "notturn" | "notplay"
}

export interface SimulateResult {
  returnValue: unknown;
  transactionXdr: string;
  ledger: number;
}

export interface SdkCoreOptions {
  rpcUrl: string;
  contractId: string;
  networkPassphrase: string;
}

export interface InvokeOptions {
  signerAddress: string;
  authEntries?: readonly string[];  // base64 XDR SorobanAuthorizationEntry
}

// ── sdk-core client stub ───────────────────────────────────────────────────

const MOCK_TX_HASH =
  'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890';

/**
 * Fixture-only stand-in for the real TurnBasedClient that will ship with
 * cougr-sdk-core (#350).  Non-fixture calls throw intentionally — this stub
 * is not a partial implementation, just enough for CI.
 */
export class TurnBasedClient {
  readonly #opts: SdkCoreOptions;
  readonly #fixture: boolean;

  constructor(opts: SdkCoreOptions, fixture = false) {
    this.#opts = opts;
    this.#fixture = fixture;
  }

  async simulate(
    method: string,
    args: readonly unknown[],
    _opts: InvokeOptions,
  ): Promise<SimulateResult> {
    void args;
    void this.#opts;
    if (this.#fixture) return mockSimulate(method);
    throw new Error(
      'TurnBasedClient.simulate: real RPC not yet implemented — ' +
      'import from cougr-sdk-core once PR #350 lands.',
    );
  }

  async submit(_signedXdr: string): Promise<string> {
    if (this.#fixture) return MOCK_TX_HASH;
    throw new Error(
      'TurnBasedClient.submit: real RPC not yet implemented — ' +
      'import from cougr-sdk-core once PR #350 lands.',
    );
  }

  async getState(_opts: Pick<InvokeOptions, 'signerAddress'>): Promise<GameState> {
    if (this.#fixture) return structuredClone(FIXTURE_INITIAL_STATE);
    throw new Error(
      'TurnBasedClient.getState: real RPC not yet implemented — ' +
      'import from cougr-sdk-core once PR #350 lands.',
    );
  }
}

export function decodeMoveResult(raw: unknown): MoveResult {
  if (
    typeof raw === 'object' &&
    raw !== null &&
    'success' in raw &&
    'game_state' in raw &&
    'message' in raw
  ) {
    return raw as MoveResult;
  }
  throw new TypeError('decodeMoveResult: unexpected shape');
}

export function decodeGameState(raw: unknown): GameState {
  if (
    typeof raw === 'object' &&
    raw !== null &&
    'cells' in raw &&
    Array.isArray((raw as { cells: unknown }).cells)
  ) {
    return raw as GameState;
  }
  throw new TypeError('decodeGameState: unexpected shape');
}

// ── sdk-core fixture constants ─────────────────────────────────────────────

export const FIXTURE_PLAYER_X =
  'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN';
export const FIXTURE_PLAYER_O =
  'GBCFAMVYPVP3C7AIZGHOWB4BPJZK3BBI4QSE53OHKZFNFB7OT6YBSIX';

export const FIXTURE_INITIAL_STATE: GameState = {
  cells: [0, 0, 0, 0, 0, 0, 0, 0, 0],
  player_x: FIXTURE_PLAYER_X,
  player_o: FIXTURE_PLAYER_O,
  is_x_turn: true,
  move_count: 0,
  status: 0,
};

export const FIXTURE_AFTER_MOVE_STATE: GameState = {
  cells: [1, 0, 0, 0, 0, 0, 0, 0, 0],  // X placed at cell 0
  player_x: FIXTURE_PLAYER_X,
  player_o: FIXTURE_PLAYER_O,
  is_x_turn: false,
  move_count: 1,
  status: 0,
};

// ── sdk-session types ──────────────────────────────────────────────────────

export type AuthEntryXdr = string;

export interface SessionScope {
  contractIds: readonly string[];
  functionNames?: readonly string[];
}

export interface SessionPolicy {
  playerAddress: string;
  sessionKeySeed: string;
  expiryLedger: number;
  scope: SessionScope;
}

export interface SessionAuthEntries {
  entries: readonly AuthEntryXdr[];
  ledger: number;
  sessionPublicKey: string;
}

// ── sdk-session builder stub ───────────────────────────────────────────────

/**
 * Fixture-only stand-in for the real SessionBuilder that will ship with
 * cougr-sdk-session (#351).
 */
export class SessionBuilder {
  #playerAddress: string;
  #sessionKeySeed: string = 'ephemeral-seed-placeholder-' + Date.now().toString(16);
  #expiryLedger: number = 0;
  #scope: SessionScope = { contractIds: [] };

  constructor(playerAddress: string) {
    this.#playerAddress = playerAddress;
  }

  withScope(scope: SessionScope): this {
    this.#scope = scope;
    return this;
  }

  withExpiry(expiryLedger: number): this {
    this.#expiryLedger = expiryLedger;
    return this;
  }

  withSeed(seed: string): this {
    this.#sessionKeySeed = seed;
    return this;
  }

  build(): SessionPolicy {
    if (this.#scope.contractIds.length === 0) {
      throw new Error('SessionBuilder: at least one contractId must be in scope');
    }
    if (this.#expiryLedger <= 0) {
      throw new Error('SessionBuilder: expiryLedger must be a positive ledger number');
    }
    return {
      playerAddress: this.#playerAddress,
      sessionKeySeed: this.#sessionKeySeed,
      expiryLedger: this.#expiryLedger,
      scope: this.#scope,
    };
  }
}

/**
 * Fixture-only stand-in for the real buildSessionAuth that will ship with
 * cougr-sdk-session (#351).  Non-fixture calls throw intentionally.
 */
export function buildSessionAuth(
  policy: SessionPolicy,
  options: {
    contractId: string;
    functionName: string;
    ledger: number;
    fixture?: boolean;
  },
): SessionAuthEntries {
  const { contractId, functionName, ledger, fixture = false } = options;

  if (!policy.scope.contractIds.includes(contractId)) {
    throw new Error(
      `buildSessionAuth: contractId ${contractId} is not in session scope`,
    );
  }
  if (
    policy.scope.functionNames !== undefined &&
    !policy.scope.functionNames.includes(functionName)
  ) {
    throw new Error(
      `buildSessionAuth: functionName ${functionName} is not in session scope`,
    );
  }
  if (ledger >= policy.expiryLedger) {
    throw new Error(
      `buildSessionAuth: session has expired (ledger ${ledger} >= expiry ${policy.expiryLedger})`,
    );
  }

  if (fixture) {
    const entry = btoa(
      `session:${policy.playerAddress}:${contractId}:${functionName}:${policy.expiryLedger}`,
    );
    return {
      entries: [entry],
      ledger,
      sessionPublicKey: `MOCK_PUBKEY_${policy.sessionKeySeed.slice(0, 8)}`,
    };
  }

  throw new Error(
    'buildSessionAuth: real XDR signing not yet implemented — ' +
    'import from cougr-sdk-session once PR #351 lands.',
  );
}

// ── sdk-session fixture constant ───────────────────────────────────────────

export const FIXTURE_SESSION_SEED = 'test-ephemeral-seed-0000';

// ── Internal ───────────────────────────────────────────────────────────────

function mockSimulate(method: string): SimulateResult {
  const returnValue =
    method === 'init_game'
      ? structuredClone(FIXTURE_INITIAL_STATE)
      : method === 'make_move'
        ? ({ success: true, game_state: structuredClone(FIXTURE_AFTER_MOVE_STATE), message: 'ok' } satisfies MoveResult)
        : null;
  return {
    returnValue,
    transactionXdr: `MOCK_XDR_${method.toUpperCase()}`,
    ledger: 1000,
  };
}
