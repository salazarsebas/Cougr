import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  replayFromEvents,
  replayFromFixture,
  replayFromPage,
  STATUS_X_WINS,
  type MatchFixtureFile,
} from '../src/index.ts';

const fixtureJson = JSON.parse(
  readFileSync(new URL('../fixtures/turn_based_match.json', import.meta.url), 'utf8'),
) as MatchFixtureFile;

test('replayFromFixture decodes full turn-based match from fixture', async () => {
  const timeline = await replayFromFixture(fixtureJson);

  assert.equal(timeline.length, 14);

  // Initial state (step 0 - TurnState opening)
  const f0 = timeline.frames[0]!;
  assert.equal(f0.ledger, 100);
  assert.equal(f0.update.family, 'set');
  assert.equal(f0.update.componentType, 'turnst');
  assert.equal(f0.state.turnState?.moveCount, 0);
  assert.equal(f0.state.turnState?.isXTurn, true);

  // Players resolved (step 1)
  const f1 = timeline.frames[1]!;
  assert.equal(f1.update.family, 'rich');
  assert.equal(f1.update.componentType, 'players');
  assert.ok(f1.state.players?.playerX?.startsWith('GBALICE'));
  assert.ok(f1.state.players?.playerO?.startsWith('GBBOB'));

  // Move 1 (step 3 - X places at cell 0)
  const f3 = timeline.frames[3]!;
  assert.equal(f3.ledger, 101);
  assert.equal(f3.state.turnState?.moveCount, 1);
  assert.equal(f3.state.turnState?.isXTurn, false);

  // Move 5 (step 11 - X places at cell 2 and wins)
  const f11 = timeline.frames[11]!;
  assert.equal(f11.ledger, 105);
  assert.equal(f11.state.turnState?.moveCount, 5);
  assert.equal(f11.state.turnState?.status, STATUS_X_WINS);
  assert.equal(f11.state.turnState?.statusLabel, 'x_wins');

  // Verify winning board
  const f12 = timeline.frames[12]!;
  assert.deepEqual(
    f12.state.board,
    [1, 1, 1, 0, 2, 2, 0, 0, 0],
  );

  // Cleanup del event (step 13)
  const f13 = timeline.frames[13]!;
  assert.equal(f13.update.family, 'del');
  assert.equal(f13.update.componentType, 'turnst');
  assert.equal(f13.state.turnState, undefined);

  // Match summary
  const summary = timeline.getSummary();
  assert.equal(summary.totalSteps, 14);
  assert.equal(summary.startLedger, 100);
  assert.equal(summary.endLedger, 106);
});

test('replayFromEvents handles out-of-order events stably', async () => {
  // Shuffle events
  const shuffled = [...fixtureJson.events].reverse();
  const timeline = await replayFromEvents(shuffled);

  // Replay timeline must still sort stably by ledger
  assert.equal(timeline.length, 14);
  assert.equal(timeline.frames[0]?.ledger, 100);
  assert.equal(timeline.frames[timeline.length - 1]?.ledger, 106);
});

test('replayFromPage decodes page using decodeCougrPage', async () => {
  const page = {
    events: fixtureJson.events.slice(0, 3),
    latestLedger: 100,
    cursor: '100-03',
  };
  const timeline = await replayFromPage(page);
  assert.equal(timeline.length, 3);
  assert.equal(timeline.frames[0]?.update.family, 'set');
  assert.equal(timeline.frames[1]?.update.family, 'rich');
});
