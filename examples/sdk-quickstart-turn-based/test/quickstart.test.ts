/**
 * Integration test for the turn-based SDK quickstart.
 *
 * Exercises the three-package composition in fixture mode:
 *   - sdk-core (stub in ./stubs.ts): TurnBasedClient + decodeMoveResult + decodeGameState
 *   - sdk-session (stub in ./stubs.ts): SessionBuilder + buildSessionAuth
 *   - cougr-sdk-events (real package): decodeCougrEvent, pageCougrEvents, resolveRichComponentUpdate
 *
 * When cougr-sdk-core (#350) and cougr-sdk-session (#351) land on main,
 * replace the stubs import with the real package imports and delete stubs.ts.
 *
 * CI entry point: `node --test test/*.test.ts`
 */

import { describe, it, before } from 'node:test';
import assert from 'node:assert/strict';

// sdk-events — real package
import {
  cougrEventFilter,
  decodeCougrEvent,
  decodeCougrEvents,
  isRichComponentChanged,
  pageCougrEvents,
  resolveRichComponentUpdate,
  serializeCursor,
  parseCursor,
  buildCougrTopicFilters,
} from 'cougr-sdk-events';

// sdk-core + sdk-session — local stubs (replace with real packages after #350 / #351 land)
import {
  TurnBasedClient,
  decodeMoveResult,
  decodeGameState,
  FIXTURE_PLAYER_X,
  FIXTURE_PLAYER_O,
  FIXTURE_INITIAL_STATE,
  FIXTURE_AFTER_MOVE_STATE,
  SessionBuilder,
  buildSessionAuth,
  FIXTURE_SESSION_SEED,
} from '../src/stubs.ts';

// game-state helpers
import {
  decodeTurnState,
  applyTurnStateUpdate,
  applyUpdatesToGameState,
  renderBoard,
  statusLabel,
  TURN_STATE_COMPONENT,
  BOARD_COMPONENT,
  STATUS_IN_PROGRESS,
} from '../src/game-state.ts';

// mock RPC
import {
  makeMockRpc,
  FIXTURE_CONTRACT_ID,
  FIXTURE_START_LEDGER,
  FIXTURE_DATA,
} from '../src/mock-rpc.ts';

import type { GameState } from '../src/game-state.ts';
import type { ComponentSetUpdate } from 'cougr-sdk-events';

// ── Helpers ───────────────────────────────────────────────────────────────

const CONTRACT_ID = FIXTURE_CONTRACT_ID;

function makeClient(): TurnBasedClient {
  return new TurnBasedClient(
    {
      rpcUrl: 'https://soroban-testnet.stellar.org',
      contractId: CONTRACT_ID,
      networkPassphrase: 'Test SDF Network ; September 2015',
    },
    true, // fixture mode
  );
}

// ── sdk-core stub tests ───────────────────────────────────────────────────

describe('sdk-core (stub)', () => {
  it('getState returns the fixture initial GameState', async () => {
    const client = makeClient();
    const state = await client.getState({ signerAddress: FIXTURE_PLAYER_X });
    assert.equal(state.move_count, 0);
    assert.equal(state.is_x_turn, true);
    assert.equal(state.status, STATUS_IN_PROGRESS);
    assert.equal(state.cells.length, 9);
    assert.ok(state.cells.every((c) => c === 0));
  });

  it('simulate returns a SimulateResult for init_game', async () => {
    const client = makeClient();
    const result = await client.simulate('init_game', [FIXTURE_PLAYER_X, FIXTURE_PLAYER_O], {
      signerAddress: FIXTURE_PLAYER_X,
    });
    assert.ok(typeof result.transactionXdr === 'string');
    assert.ok(result.transactionXdr.length > 0);
    assert.ok(typeof result.ledger === 'number');
    assert.ok(result.ledger > 0);
  });

  it('simulate returns a MoveResult for make_move', async () => {
    const client = makeClient();
    const result = await client.simulate('make_move', [FIXTURE_PLAYER_X, 0], {
      signerAddress: FIXTURE_PLAYER_X,
    });
    const moveResult = decodeMoveResult(result.returnValue);
    assert.equal(moveResult.success, true);
    assert.equal(moveResult.message, 'ok');
    assert.equal(moveResult.game_state.move_count, 1);
    assert.equal(moveResult.game_state.cells[0], 1); // X placed at cell 0
  });

  it('submit returns a tx hash string', async () => {
    const client = makeClient();
    const hash = await client.submit('MOCK_XDR');
    assert.ok(typeof hash === 'string' && hash.length > 0);
  });

  it('decodeMoveResult throws on invalid shape', () => {
    assert.throws(() => decodeMoveResult(null), /decodeMoveResult/);
    assert.throws(() => decodeMoveResult(42), /decodeMoveResult/);
  });

  it('decodeGameState throws on invalid shape', () => {
    assert.throws(() => decodeGameState(null), /decodeGameState/);
    assert.throws(() => decodeGameState({ noFields: true }), /decodeGameState/);
  });

  it('FIXTURE_INITIAL_STATE and FIXTURE_AFTER_MOVE_STATE have expected shapes', () => {
    assert.equal(FIXTURE_INITIAL_STATE.move_count, 0);
    assert.ok(FIXTURE_INITIAL_STATE.cells.every((c) => c === 0));
    assert.equal(FIXTURE_AFTER_MOVE_STATE.move_count, 1);
    assert.equal(FIXTURE_AFTER_MOVE_STATE.cells[0], 1);
  });
});

// ── sdk-session stub tests ────────────────────────────────────────────────

describe('sdk-session (stub)', () => {
  it('SessionBuilder produces a valid SessionPolicy', () => {
    const policy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: [CONTRACT_ID], functionNames: ['make_move'] })
      .withExpiry(2000)
      .build();

    assert.equal(policy.playerAddress, FIXTURE_PLAYER_X);
    assert.equal(policy.sessionKeySeed, FIXTURE_SESSION_SEED);
    assert.deepEqual(policy.scope.contractIds, [CONTRACT_ID]);
    assert.deepEqual(policy.scope.functionNames, ['make_move']);
    assert.equal(policy.expiryLedger, 2000);
  });

  it('SessionBuilder throws without contractIds', () => {
    assert.throws(
      () =>
        new SessionBuilder(FIXTURE_PLAYER_X)
          .withScope({ contractIds: [] })
          .withExpiry(2000)
          .build(),
      /contractId/,
    );
  });

  it('SessionBuilder throws without expiryLedger', () => {
    assert.throws(
      () =>
        new SessionBuilder(FIXTURE_PLAYER_X)
          .withScope({ contractIds: [CONTRACT_ID] })
          .build(),
      /expiryLedger/,
    );
  });

  it('buildSessionAuth returns auth entries in fixture mode', () => {
    const policy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: [CONTRACT_ID], functionNames: ['make_move'] })
      .withExpiry(2000)
      .build();

    const auth = buildSessionAuth(policy, {
      contractId: CONTRACT_ID,
      functionName: 'make_move',
      ledger: 1000,
      fixture: true,
    });

    assert.ok(Array.isArray(auth.entries));
    assert.equal(auth.entries.length, 1);
    assert.ok(typeof auth.entries[0] === 'string' && auth.entries[0].length > 0);
    assert.ok(typeof auth.sessionPublicKey === 'string');
    assert.equal(auth.ledger, 1000);
  });

  it('buildSessionAuth throws when contractId not in scope', () => {
    const policy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: ['OTHER_CONTRACT'] })
      .withExpiry(2000)
      .build();

    assert.throws(
      () =>
        buildSessionAuth(policy, {
          contractId: CONTRACT_ID,
          functionName: 'make_move',
          ledger: 1000,
          fixture: true,
        }),
      /not in session scope/,
    );
  });

  it('buildSessionAuth throws when functionName not in scope', () => {
    const policy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: [CONTRACT_ID], functionNames: ['other_fn'] })
      .withExpiry(2000)
      .build();

    assert.throws(
      () =>
        buildSessionAuth(policy, {
          contractId: CONTRACT_ID,
          functionName: 'make_move',
          ledger: 1000,
          fixture: true,
        }),
      /not in session scope/,
    );
  });

  it('buildSessionAuth throws when session is expired', () => {
    const policy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: [CONTRACT_ID] })
      .withExpiry(500)
      .build();

    assert.throws(
      () =>
        buildSessionAuth(policy, {
          contractId: CONTRACT_ID,
          functionName: 'make_move',
          ledger: 1000, // >= expiry 500
          fixture: true,
        }),
      /expired/,
    );
  });
});

// ── sdk-events tests (real package) ──────────────────────────────────────

describe('sdk-events: decodeCougrEvent on fixture events', () => {
  it('loads and decodes the sdk-events package fixture events', async () => {
    // Use the existing sdk-events fixtures to verify the real package works.
    // These are the three events from packages/sdk-events/fixtures/events.json.
    const sdkEventsFixturePath = new URL(
      '../../../packages/sdk-events/fixtures/events.json',
      import.meta.url,
    );
    const { readFileSync } = await import('node:fs');
    const raw = JSON.parse(readFileSync(sdkEventsFixturePath, 'utf8')) as {
      events: Array<{ kind: string; event: unknown }>;
    };

    const updates = decodeCougrEvents(raw.events.map((e) => e.event as Parameters<typeof decodeCougrEvent>[0]));
    assert.equal(updates.length, 3);
    assert.equal(updates[0]?.family, 'set');
    assert.equal(updates[1]?.family, 'del');
    assert.equal(updates[2]?.family, 'rich');
  });

  it('decodes the quickstart fixture events', () => {
    const events = FIXTURE_DATA.events.map((e) => e.event);
    const updates = decodeCougrEvents(events);
    // turnst (set) + board (rich)
    assert.equal(updates.length, 2);
    const setUpdate = updates[0];
    const richUpdate = updates[1];
    assert.ok(setUpdate !== undefined);
    assert.ok(richUpdate !== undefined);
    assert.equal(setUpdate.family, 'set');
    assert.equal(setUpdate.componentType, TURN_STATE_COMPONENT);
    assert.equal(richUpdate.family, 'rich');
    assert.equal(richUpdate.componentType, BOARD_COMPONENT);
  });
});

describe('sdk-events: pageCougrEvents + cursor', () => {
  it('pages the mock RPC and returns fixture updates', async () => {
    const getEvents = makeMockRpc();
    const filter = cougrEventFilter({ contractIds: [CONTRACT_ID] });
    const gen = pageCougrEvents(getEvents, {
      filters: [filter],
      startLedger: FIXTURE_START_LEDGER,
      maxPages: 1,
    });

    const { value: page } = await gen.next();
    assert.ok(page !== undefined);
    assert.equal(page.updates.length, 2);
    assert.equal(page.updates[0]?.family, 'set');
    assert.equal(page.updates[1]?.family, 'rich');
  });

  it('cursor can be serialized and parsed', async () => {
    const getEvents = makeMockRpc();
    const filter = cougrEventFilter({ contractIds: [CONTRACT_ID] });
    const gen = pageCougrEvents(getEvents, {
      filters: [filter],
      startLedger: FIXTURE_START_LEDGER,
      maxPages: 1,
    });

    const { value: page } = await gen.next();
    assert.ok(page !== undefined);

    const serialized = serializeCursor(page.cursor);
    const restored = parseCursor(serialized);
    assert.equal(restored.pagingToken, page.cursor.pagingToken);
    assert.equal(restored.ledger, page.cursor.ledger);
  });
});

describe('sdk-events: resolveRichComponentUpdate', () => {
  it('resolves the board rich update via a follow-up reader', async () => {
    const events = FIXTURE_DATA.events.map((e) => e.event);
    const updates = decodeCougrEvents(events);
    const richUpdate = updates.find(isRichComponentChanged);
    assert.ok(richUpdate !== undefined, 'expected a rich update');
    assert.equal(richUpdate.followUp.componentType, BOARD_COMPONENT);

    const { value } = await resolveRichComponentUpdate(
      richUpdate,
      async ({ componentType }) => {
        if (componentType === BOARD_COMPONENT) {
          return { cells: [1, 0, 0, 0, 0, 0, 0, 0, 0] };
        }
        throw new Error(`unexpected component: ${componentType}`);
      },
    );
    assert.deepEqual((value as { cells: number[] }).cells, [1, 0, 0, 0, 0, 0, 0, 0, 0]);
  });

  it('cougrEventFilter produces a valid filter', () => {
    const filter = cougrEventFilter({ contractIds: [CONTRACT_ID] });
    // buildCougrTopicFilters takes CougrTopicFilterOptions (no contractIds)
    const filters = buildCougrTopicFilters({ families: ['set', 'del', 'rich'] });
    assert.ok(filter !== null && filter !== undefined);
    assert.ok(Array.isArray(filters));
  });
});

// ── game-state tests ───────────────────────────────────────────────────────

describe('game-state: decodeTurnState', () => {
  it('decodes a 9-byte TurnState buffer', () => {
    // Layout: [is_x_turn: bool, move_count: u32 big-endian, status: u32 big-endian]
    // move_count=1, is_x_turn=false, status=0
    const buf = new Uint8Array(9);
    buf[0] = 0;                              // is_x_turn = false
    new DataView(buf.buffer).setUint32(1, 1, false); // move_count = 1
    new DataView(buf.buffer).setUint32(5, 0, false); // status = 0
    const decoded = decodeTurnState(buf);
    assert.equal(decoded.isXTurn, false);
    assert.equal(decoded.moveCount, 1);
    assert.equal(decoded.status, 0);
    assert.equal(decoded.statusLabel, 'in_progress');
  });

  it('throws on a buffer shorter than 9 bytes', () => {
    assert.throws(() => decodeTurnState(new Uint8Array(5)), /9/);
  });
});

describe('game-state: applyUpdatesToGameState', () => {
  let initialState: GameState;

  before(() => {
    initialState = { ...FIXTURE_INITIAL_STATE };
  });

  it('applies a TurnState set-update without a get_state poll', () => {
    const events = FIXTURE_DATA.events.map((e) => e.event);
    const updates = decodeCougrEvents(events);

    const { state, richPending } = applyUpdatesToGameState(initialState, updates);

    // TurnState was updated from the event
    assert.equal(state.move_count, initialState.move_count + 1);
    // We intentionally do NOT assert the exact value because the mock TurnState
    // bytes in the fixture file use stub XDR: real Soroban bytes would differ.
    // What we assert is that the function ran without error and produced a
    // state with the same player addresses as before.
    assert.equal(state.player_x, initialState.player_x);
    assert.equal(state.player_o, initialState.player_o);

    // Board update deferred to rich-pending
    assert.ok(richPending.length > 0);
    assert.ok(richPending.every((u) => u.family === 'rich'));
  });

  it('does not modify original state object', () => {
    const events = FIXTURE_DATA.events.map((e) => e.event);
    const updates = decodeCougrEvents(events);
    const clone = { ...initialState };
    applyUpdatesToGameState(initialState, updates);
    assert.deepEqual(initialState, clone);
  });
});

describe('game-state: renderBoard', () => {
  it('renders a 3x3 board with X at position 0', () => {
    const cells = [1, 0, 0, 0, 0, 0, 0, 0, 0];
    const rendered = renderBoard(cells);
    assert.ok(rendered.includes('X'));
    assert.ok(rendered.includes('.'));
    assert.ok(rendered.includes('|'));
  });
});

// ── End-to-end composition test ───────────────────────────────────────────

describe('end-to-end: three-package composition (sdk-core stub + sdk-session stub + sdk-events)', () => {
  it('composes without a live node', async () => {
    // 1. sdk-core (stub)
    const client = makeClient();
    const simResult = await client.simulate('init_game', [FIXTURE_PLAYER_X, FIXTURE_PLAYER_O], {
      signerAddress: FIXTURE_PLAYER_X,
    });
    let gameState = await client.getState({ signerAddress: FIXTURE_PLAYER_X });

    // 2. sdk-session (stub)
    const sessionPolicy = new SessionBuilder(FIXTURE_PLAYER_X)
      .withSeed(FIXTURE_SESSION_SEED)
      .withScope({ contractIds: [CONTRACT_ID], functionNames: ['make_move'] })
      .withExpiry(simResult.ledger + 200)
      .build();

    const sessionAuth = buildSessionAuth(sessionPolicy, {
      contractId: CONTRACT_ID,
      functionName: 'make_move',
      ledger: simResult.ledger,
      fixture: true,
    });
    assert.ok(sessionAuth.entries.length > 0, 'session auth entries present');

    // 3. sdk-core (stub) makes the move with session auth
    const moveSim = await client.simulate('make_move', [FIXTURE_PLAYER_X, 0], {
      signerAddress: FIXTURE_PLAYER_X,
      authEntries: sessionAuth.entries,
    });
    await client.submit(moveSim.transactionXdr);
    const moveResult = decodeMoveResult(moveSim.returnValue);
    assert.equal(moveResult.success, true);

    // 4. sdk-events (real) decodes the resulting events
    const getEvents = makeMockRpc();
    const filter = cougrEventFilter({ contractIds: [CONTRACT_ID] });
    const gen = pageCougrEvents(getEvents, {
      filters: [filter],
      startLedger: FIXTURE_START_LEDGER,
      maxPages: 1,
    });
    const { value: page } = await gen.next();
    assert.ok(page !== undefined, 'event page received');

    // 5. Apply turn-state update to GameState WITHOUT a second get_state poll
    const { state: updatedState, richPending } = applyUpdatesToGameState(
      gameState,
      page.updates,
    );
    gameState = updatedState;

    // Assert TurnState was reconstructed from the event
    const turnStateUpdatesCount = page.updates.filter(
      (u) => u.family === 'set' && u.componentType === TURN_STATE_COMPONENT,
    ).length;
    assert.ok(turnStateUpdatesCount > 0, 'TurnState set-update present in events');
    assert.ok(richPending.some((u) => u.componentType === BOARD_COMPONENT),
      'Board rich-pending present');

    // 6. Resolve rich-component update for Board
    const richUpdate = richPending.find(isRichComponentChanged);
    assert.ok(richUpdate !== undefined, 'rich update for board found');
    const { value: boardValue } = await resolveRichComponentUpdate(richUpdate, async () => ({
      cells: [1, 0, 0, 0, 0, 0, 0, 0, 0],
    }));
    gameState = { ...gameState, cells: (boardValue as { cells: number[] }).cells };

    // 7. Final assertions
    assert.equal(gameState.cells[0], 1, 'X placed at cell 0 after board resolved');
    assert.equal(gameState.player_x, FIXTURE_PLAYER_X);
    assert.equal(gameState.player_o, FIXTURE_PLAYER_O);
  });
});
