/**
 * sdk-quickstart-turn-based
 *
 * Demonstrates the three-package composition against the turn-based contract:
 *
 *   sdk-core  (#324 / PR #350 pending) — simulate + submit + decode
 *   sdk-session (#325 / PR #351 pending) — session-scoped authorization
 *   sdk-events (#327, merged) — COUGR event decode, no follow-up get_state poll
 *
 * Until PR #350 (cougr-sdk-core) and PR #351 (cougr-sdk-session) land on
 * main, the sdk-core and sdk-session surfaces are provided by ./stubs.ts.
 * When those PRs merge, replace the stubs import with:
 *
 *   import { TurnBasedClient, decodeMoveResult, ... } from 'cougr-sdk-core';
 *   import { SessionBuilder, buildSessionAuth, ... } from 'cougr-sdk-session';
 *
 * This script is intentionally a Node.js script, not a browser app.  It has
 * no wallet and no UI; its only job is to prove the three packages compose.
 *
 * Run it:
 *   FIXTURE_MODE=true node src/quickstart.ts
 *
 * Architecture:
 *   1. sdk-core simulates `init_game` and returns the initial GameState.
 *   2. sdk-session builds a scoped SessionPolicy and produces auth entries
 *      for the upcoming `make_move` call.
 *   3. sdk-core simulates `make_move` with those auth entries attached, then
 *      submits the pre-signed transaction.
 *   4. sdk-events pages the resulting COUGR events from the (mock) RPC.
 *   5. The `turnst` set-update is binary-decoded and merged into GameState —
 *      no second `get_state` call is made.
 *   6. The `board` rich-update's `requiresFollowUpRead: true` field forces
 *      an explicit follow-up read, which is performed here.
 *
 * Signer ownership (matches each package's own issue scope):
 *   - sdk-core owns transport: simulate, sign, submit.
 *   - sdk-session owns the session keypair: it produces auth entries that
 *     sdk-core attaches to the transaction.
 *   - sdk-events owns event decoding: it has no signer at all.
 */

import {
  cougrEventFilter,
  isRichComponentChanged,
  pageCougrEvents,
  resolveRichComponentUpdate,
  serializeCursor,
} from 'cougr-sdk-events';

// TODO(#350, #351): replace with real package imports once those PRs land:
//   import { TurnBasedClient, decodeMoveResult, FIXTURE_PLAYER_X, FIXTURE_PLAYER_O } from 'cougr-sdk-core';
//   import { SessionBuilder, buildSessionAuth, FIXTURE_SESSION_SEED } from 'cougr-sdk-session';
import {
  TurnBasedClient,
  decodeMoveResult,
  FIXTURE_PLAYER_X,
  FIXTURE_PLAYER_O,
  SessionBuilder,
  buildSessionAuth,
  FIXTURE_SESSION_SEED,
} from './stubs.ts';

import {
  applyUpdatesToGameState,
  renderBoard,
  statusLabel,
  TURN_STATE_COMPONENT,
} from './game-state.ts';

import {
  makeMockRpc,
  FIXTURE_CONTRACT_ID,
  FIXTURE_START_LEDGER,
} from './mock-rpc.ts';

import type { GameState } from './game-state.ts';

// ── Configuration ─────────────────────────────────────────────────────────

const CONTRACT_ID = FIXTURE_CONTRACT_ID;
const RPC_URL = 'https://soroban-testnet.stellar.org'; // unused in fixture mode
const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';

// When FIXTURE_MODE=false and a real RPC/contract are configured, this script
// runs against a live testnet.  CI always uses FIXTURE_MODE=true.
const FIXTURE_MODE = process.env['FIXTURE_MODE'] !== 'false';

// ── Main ──────────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  console.log('sdk-quickstart-turn-based');
  console.log('='.repeat(50));
  console.log(`Mode   : ${FIXTURE_MODE ? 'fixture (no live testnet)' : 'live'}`);
  console.log(`Contract: ${CONTRACT_ID}`);
  console.log();

  // ── 1. sdk-core: instantiate the client ────────────────────────────────
  //
  // sdk-core owns transport. It knows the RPC URL, contract, and network
  // passphrase. It does not own keys or auth entries.

  const sdkCore = new TurnBasedClient(
    { rpcUrl: RPC_URL, contractId: CONTRACT_ID, networkPassphrase: NETWORK_PASSPHRASE },
    FIXTURE_MODE,
  );

  // ── 2. sdk-core: simulate init_game ────────────────────────────────────
  //
  // `init_game` does not require a session key: it is called once by the
  // player's own wallet. sdk-core simulates it and returns the initial
  // GameState via `decodeMoveResult` / `decodeGameState`.

  console.log('Step 1 — sdk-core: simulate init_game');
  const initSim = await sdkCore.simulate('init_game', [FIXTURE_PLAYER_X, FIXTURE_PLAYER_O], {
    signerAddress: FIXTURE_PLAYER_X,
  });
  console.log(`  simulation ledger : ${initSim.ledger}`);

  let gameState: GameState = await sdkCore.getState({ signerAddress: FIXTURE_PLAYER_X });
  console.log(`  initial state     : move_count=${gameState.move_count}  is_x_turn=${gameState.is_x_turn}  status=${statusLabel(gameState.status)}`);
  console.log();

  // ── 3. sdk-session: build a session policy ─────────────────────────────
  //
  // sdk-session owns the session keypair. The builder accepts the player's
  // address, a scope (which contracts and functions the session may sign),
  // and an expiry ledger. `build()` produces a `SessionPolicy` object.
  //
  // The player's main wallet approves the session once (off-chain ceremony
  // described in #325's own scope). Here we use a fixture seed so CI output
  // is reproducible.

  console.log('Step 2 — sdk-session: build session policy for make_move');
  const sessionPolicy = new SessionBuilder(FIXTURE_PLAYER_X)
    .withSeed(FIXTURE_SESSION_SEED)
    .withScope({
      contractIds: [CONTRACT_ID],
      functionNames: ['make_move'],
    })
    .withExpiry(initSim.ledger + 200)   // valid for ~200 ledgers (~20 min on testnet)
    .build();

  console.log(`  player            : ${sessionPolicy.playerAddress.slice(0, 10)}…`);
  console.log(`  scope             : ${sessionPolicy.scope.contractIds[0]?.slice(0, 10)}…  fns: ${sessionPolicy.scope.functionNames?.join(', ')}`);
  console.log(`  expiry ledger     : ${sessionPolicy.expiryLedger}`);
  console.log();

  // ── 4. sdk-session: produce auth entries for this specific call ─────────
  //
  // `buildSessionAuth` signs the invocation description with the session
  // keypair and returns base64-encoded `SorobanAuthorizationEntry` XDR blobs.
  // sdk-core attaches these to the assembled transaction — it does not derive
  // or hold any key material.

  console.log('Step 3 — sdk-session: build auth entries for make_move(player_x, 0)');
  const sessionAuth = buildSessionAuth(sessionPolicy, {
    contractId: CONTRACT_ID,
    functionName: 'make_move',
    ledger: initSim.ledger,
    fixture: FIXTURE_MODE,
  });
  console.log(`  auth entries      : ${sessionAuth.entries.length}`);
  console.log(`  session pubkey    : ${sessionAuth.sessionPublicKey}`);
  console.log(`  entry[0] prefix   : ${(sessionAuth.entries[0] ?? '').slice(0, 20)}…`);
  console.log();

  // ── 5. sdk-core: simulate and submit make_move with session auth ────────
  //
  // sdk-core simulates `make_move` with the session's auth entries attached.
  // On a live network it would also sign with the fee-payer key and submit.
  // In fixture mode it returns a pre-built MoveResult immediately.

  console.log('Step 4 — sdk-core: simulate + submit make_move (position=0, session-signed)');
  const moveSim = await sdkCore.simulate(
    'make_move',
    [FIXTURE_PLAYER_X, 0],
    { signerAddress: FIXTURE_PLAYER_X, authEntries: sessionAuth.entries },
  );
  const txHash = await sdkCore.submit(moveSim.transactionXdr);
  const moveResult = decodeMoveResult(moveSim.returnValue);

  console.log(`  tx hash           : ${txHash}`);
  console.log(`  move success      : ${moveResult.success}`);
  console.log(`  message           : ${moveResult.message}`);
  console.log();

  // ── 6. sdk-events: page the resulting COUGR events ─────────────────────
  //
  // Rather than calling get_state again, we decode the events the contract
  // emitted and apply them directly to the last-known GameState.
  //
  // The mock RPC returns the fixture events: one `("COUGR","set","turnst")`
  // carrying the new TurnState bytes, and one `("COUGR","rich","board")`
  // indicating the Board changed (requiring a follow-up read).

  console.log('Step 5 — sdk-events: page COUGR events from the make_move ledger');
  const getEvents = makeMockRpc();
  const filter = cougrEventFilter({ contractIds: [CONTRACT_ID] });

  const gen = pageCougrEvents(getEvents, {
    filters: [filter],
    startLedger: FIXTURE_START_LEDGER,
    maxPages: 1,
  });

  const { value: eventPage } = await gen.next();
  if (eventPage === undefined) throw new Error('expected at least one event page');

  console.log(`  events decoded    : ${eventPage.updates.length}`);
  console.log(`  latest ledger     : ${eventPage.latestLedger}`);
  console.log(`  cursor            : ${serializeCursor(eventPage.cursor)}`);
  for (const u of eventPage.updates) {
    console.log(`    [${u.family.padEnd(3)}] componentType=${u.componentType}  entityId=${u.entityId}`);
  }
  console.log();

  // ── 7. Apply TurnState update without a follow-up get_state poll ────────
  //
  // `applyUpdatesToGameState` binary-decodes the `turnst` set-update and
  // merges it into `gameState`. This is the core claim of the issue: the new
  // `is_x_turn`, `move_count`, and `status` are known from the event alone.
  //
  // Rich-component updates (`board`, `players`) require a follow-up read —
  // the sdk-events type forces this because `RichComponentChangedUpdate` has
  // no `.data` field, only `followUp: { kind: 'contractData', ... }`.

  console.log('Step 6 — apply events to GameState (no second get_state call)');
  const prevState = gameState;
  const { state: newState, richPending } = applyUpdatesToGameState(
    prevState,
    eventPage.updates,
  );
  gameState = newState;

  console.log(`  turn-state updated from event:`);
  console.log(`    move_count  : ${prevState.move_count} → ${gameState.move_count}`);
  console.log(`    is_x_turn   : ${prevState.is_x_turn} → ${gameState.is_x_turn}`);
  console.log(`    status      : ${statusLabel(prevState.status)} → ${statusLabel(gameState.status)}`);
  console.log();

  // ── 8. Handle rich-component updates ───────────────────────────────────
  //
  // The sdk-events type system prevents silent data loss on rich components:
  // `RichComponentChangedUpdate.requiresFollowUpRead` is the literal type
  // `true`, so you cannot accidentally skip the read.
  //
  // `resolveRichComponentUpdate` takes a reader function — in production it
  // calls getLedgerEntries or your read system. Here we supply a fake reader
  // returning a fixture board so CI can assert the result without a node.

  if (richPending.length > 0) {
    console.log(`Step 7 — resolve ${richPending.length} rich-component update(s) via follow-up read`);

    for (const richUpdate of richPending) {
      if (!isRichComponentChanged(richUpdate)) continue;

      const { update, value } = await resolveRichComponentUpdate(
        richUpdate,
        async ({ componentType, entityId }) => {
          // In production: call getLedgerEntries with the storage key
          // `(cougr_rc, entity_id, component_type)`.
          if (componentType === 'board') {
            return { cells: [1, 0, 0, 0, 0, 0, 0, 0, 0] }; // X at cell 0
          }
          throw new Error(`Unknown rich component: ${componentType} (entity ${entityId})`);
        },
      );

      if (componentType(update) === 'board') {
        const resolved = value as { cells: number[] };
        gameState = { ...gameState, cells: resolved.cells };
        console.log(`  board cells resolved: [${resolved.cells.join(',')}]`);
      }
    }
    console.log();
  }

  // ── 9. Verify the TurnState component was NOT decoded from a richUpdate ──
  //
  // Validate that the key claim of the issue holds: the `turnst` update came
  // through as a `set` event (not `rich`), so the binary decoder ran and the
  // client never needed a second RPC call to learn the new turn position.

  const turnStateUpdates = eventPage.updates.filter(
    (u) => u.family === 'set' && u.componentType === TURN_STATE_COMPONENT,
  );
  if (turnStateUpdates.length === 0) {
    throw new Error(
      'ASSERTION FAILED: expected at least one TurnState set-update in events',
    );
  }

  // ── 10. Render the final board ──────────────────────────────────────────

  console.log('Final GameState (reconstructed from events, no get_state poll):');
  console.log(`  status            : ${statusLabel(gameState.status)}`);
  console.log(`  move_count        : ${gameState.move_count}`);
  console.log(`  is_x_turn         : ${gameState.is_x_turn}`);
  console.log(`  board:\n${renderBoard(gameState.cells).split('\n').map((l) => '    ' + l).join('\n')}`);
  console.log();

  console.log('✓ sdk-core (stub), sdk-session (stub), and sdk-events composed successfully.');
  console.log('  TurnState reconstructed from event bytes (no follow-up get_state poll).');
  console.log('  Board reconstructed via required rich-component follow-up read.');
  console.log('  Replace stubs.ts imports with cougr-sdk-core + cougr-sdk-session once #350 and #351 land.');
}

/** Return the componentType of any CougrEventUpdate. */
function componentType(u: { componentType: string }): string {
  return u.componentType;
}

main().catch((err: unknown) => {
  console.error('quickstart failed:', err);
  process.exitCode = 1;
});
