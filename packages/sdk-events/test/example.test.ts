import assert from 'node:assert/strict';
import test from 'node:test';

import { buildFilter, runWalkthrough } from '../examples/walkthrough.ts';

test('filter includes all three families for the given contract', () => {
  const filter = buildFilter('CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM');
  assert.equal(filter.type, 'contract');
  assert.deepEqual(filter.contractIds, ['CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM']);
  // Three topic arrays: one per family (set, del, rich)
  assert.equal(filter.topics?.length, 3);
});

test('walkthrough: page 1 yields set and del updates', async () => {
  const result = await runWalkthrough();

  assert.equal(result.page1Updates.length, 2);

  const [setUpdate, delUpdate] = result.page1Updates;
  assert.ok(setUpdate && delUpdate);

  assert.equal(setUpdate.family, 'set');
  assert.equal(setUpdate.componentType, 'position');
  assert.equal(setUpdate.entityId, 7);
  if (setUpdate.family === 'set') {
    assert.deepEqual([...setUpdate.data], [1, 2, 3, 4]);
  }

  assert.equal(delUpdate.family, 'del');
  assert.equal(delUpdate.componentType, 'position');
  assert.equal(delUpdate.entityId, 7);
  assert.equal('data' in delUpdate, false);
});

test('walkthrough: cursor serializes and resume works', async () => {
  const result = await runWalkthrough();

  const cursor = JSON.parse(result.resumedCursor) as { pagingToken: unknown; ledger: unknown };
  assert.equal(cursor.pagingToken, '100-del');
  assert.equal(typeof cursor.ledger, 'number');
});

test('walkthrough: page 2 yields a rich update (no data field)', async () => {
  const result = await runWalkthrough();

  assert.equal(result.page2Updates.length, 1);
  const [richUpdate] = result.page2Updates;
  assert.ok(richUpdate);

  assert.equal(richUpdate.family, 'rich');
  assert.equal(richUpdate.componentType, 'player_profile');
  assert.equal(richUpdate.entityId, 7);
  assert.equal('data' in richUpdate, false);

  if (richUpdate.family === 'rich') {
    assert.equal(richUpdate.requiresFollowUpRead, true);
    assert.equal(richUpdate.followUp.componentType, 'player_profile');
    assert.equal(richUpdate.followUp.entityId, 7);
  }
});

test('walkthrough: rich follow-up read resolves with the fake reader value', async () => {
  const result = await runWalkthrough();

  assert.equal(result.richResolved.componentType, 'player_profile');
  assert.equal(result.richResolved.entityId, 7);
  assert.deepEqual(result.richResolved.value, { level: 42, handle: 'player_profile:7' });
});
