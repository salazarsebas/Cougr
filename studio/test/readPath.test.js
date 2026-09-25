import test from 'node:test';
import assert from 'node:assert/strict';

import {
  READ_PATH,
  ReadPathTimeoutError,
  defaultIsSettled,
  pollForSettlement,
  readPathJustification,
} from '../src/readPath.js';
import { loadFixture } from '../tools/fixtures.js';

/** A clock that returns a scripted sequence, then stays on the last value. */
function scriptedClock(values) {
  let index = 0;
  return () => {
    const value = values[Math.min(index, values.length - 1)];
    index += 1;
    return value;
  };
}

const noWait = async () => {};

test('the declared read path is polling get_state', () => {
  assert.equal(READ_PATH.kind, 'poll');
  assert.equal(READ_PATH.contractCall, 'get_state');
  assert.deepEqual(READ_PATH.eventTopics, ['set', 'del', 'rich']);
  assert.equal(READ_PATH.richEventCarriesValue, false);
});

test('the justification names the rich-event gap and the follow-up read it forces', () => {
  const justification = readPathJustification();
  assert.equal(justification.choice, 'poll get_state');
  assert.equal(justification.followUpReadRequiredForEvents, true);
  assert.match(justification.summary, /does not carry the component value/);
  assert.ok(justification.reasons.some((reason) => reason.includes('RichComponentChangedEvent')));
  assert.ok(justification.reasons.some((reason) => reason.includes('follow-up') || reason.includes('serialization')));
  assert.ok(justification.tradeoffs.length > 0);
});

test('defaultIsSettled detects a new move and a status change, not an identical read', () => {
  const before = { cells: [], status: 0, is_x_turn: true, move_count: 2 };
  assert.equal(defaultIsSettled(before, { ...before }), false);
  assert.equal(defaultIsSettled(before, { ...before, move_count: 3 }), true);
  assert.equal(defaultIsSettled(before, { ...before, status: 1 }), true);
  assert.equal(defaultIsSettled(null, before), true);
});

test('pollForSettlement resolves as soon as the state changes', async () => {
  const fixture = loadFixture('3x3-win');
  const before = fixture.state;
  const after = { ...before, move_count: before.move_count + 1 };
  const reads = [before, before, after];
  let calls = 0;

  const result = await pollForSettlement({
    readState: async () => reads[calls++],
    previousState: before,
    pollIntervalMs: 1000,
    maxPolls: 10,
    wait: noWait,
    now: scriptedClock([0, 1000, 2000, 3000]),
  });

  assert.equal(result.polls, 3);
  assert.equal(result.state, after);
  assert.equal(calls, 3);
});

test('pollForSettlement times out instead of hanging when nothing ever changes', async () => {
  const fixture = loadFixture('3x3-win');
  const frozen = fixture.state;
  let calls = 0;

  await assert.rejects(
    pollForSettlement({
      readState: async () => { calls += 1; return frozen; },
      previousState: frozen,
      pollIntervalMs: 1000,
      maxPolls: 4,
      finalityTimeoutMs: 4000,
      wait: noWait,
      now: scriptedClock([0, 1000, 2000, 3000, 4000]),
    }),
    (error) => {
      assert.ok(error instanceof ReadPathTimeoutError);
      assert.equal(error.polls, 4);
      assert.equal(error.timeoutMs, 4000);
      return true;
    },
  );

  assert.equal(calls, 4, 'the poll loop must stop at maxPolls, not keep reading');
});

test('pollForSettlement reports progress on every read that has not settled', async () => {
  const before = { cells: [], status: 0, is_x_turn: true, move_count: 0 };
  const after = { ...before, move_count: 1 };
  const seen = [];
  let reads = 0;

  const result = await pollForSettlement({
    readState: async () => (reads++ < 2 ? before : after),
    previousState: before,
    maxPolls: 5,
    wait: noWait,
    onPoll: (_state, polls) => seen.push(polls),
  });

  assert.deepEqual(seen, [1, 2, 3], 'every read is reported, including the one that settles');
  assert.equal(result.polls, 3);
});

test('pollForSettlement lets a read error through to the caller', async () => {
  const before = { cells: [], status: 0, is_x_turn: true, move_count: 0 };

  await assert.rejects(
    pollForSettlement({
      readState: async () => { throw new Error('ECONNRESET'); },
      previousState: before,
      maxPolls: 5,
      wait: noWait,
    }),
    /ECONNRESET/,
  );
});

test('pollForSettlement requires a readState function', async () => {
  await assert.rejects(pollForSettlement({}), /requires a readState function/);
});
