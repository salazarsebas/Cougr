import test from 'node:test';
import assert from 'node:assert/strict';

import { defaultConfig } from '../src/config.js';
import { DEFAULT_POLICY, describeRunError, runMatch } from '../src/run.js';
import { loadFixture } from '../tools/fixtures.js';

const config = defaultConfig('tic_tac_toe', 3, 3);
const noWait = async () => {};

const emptyBoard = () => new Array(9).fill(0);
const previous = { cells: emptyBoard(), status: 0, is_x_turn: true, move_count: 0 };
const afterMove = { cells: [1, 0, 0, 0, 0, 0, 0, 0, 0], status: 0, is_x_turn: false, move_count: 1 };

/** A run policy with no real time in it. */
const fastPolicy = (overrides = {}) => ({
  pollIntervalMs: 1,
  finalityTimeoutMs: 4,
  wait: noWait,
  now: () => 0,
  ...overrides,
});

test('a successful run funds, submits, waits, then settles on the confirmed state', async () => {
  const phases = [];
  let funded = 0;
  let submitted = 0;

  const model = await runMatch({
    config,
    previousState: previous,
    move: { cell: 0 },
    client: {
      ensureFunded: async () => { funded += 1; },
      submitMove: async () => { submitted += 1; return { txHash: 'tx-happy' }; },
      readState: async () => afterMove,
    },
    policy: fastPolicy(),
    hooks: { onPhase: (m) => phases.push(m.phase) },
  });

  assert.equal(model.phase, 'settled');
  assert.equal(model.error, null);
  assert.equal(model.state, afterMove);
  assert.equal(model.txHash, 'tx-happy');
  assert.equal(model.polls, 1);
  assert.equal(funded, 1);
  assert.equal(submitted, 1);

  assert.equal(phases[0], 'funding');
  assert.equal(phases[1], 'submitting');
  assert.ok(phases.includes('finalizing'), 'the wait must be a visible phase');
  assert.equal(phases.at(-1), 'settled');
});

test('a friendbot failure is an error and no move is submitted', async () => {
  let submitted = 0;

  const model = await runMatch({
    config,
    previousState: previous,
    client: {
      ensureFunded: async () => { throw new Error('friendbot 429'); },
      submitMove: async () => { submitted += 1; return { txHash: 'never' }; },
      readState: async () => afterMove,
    },
    policy: fastPolicy(),
  });

  assert.equal(model.phase, 'error');
  assert.equal(model.error.kind, 'friendbot');
  assert.match(model.error.title, /funded/);
  assert.equal(model.error.detail, 'friendbot 429');
  assert.equal(model.error.retryable, true);
  assert.equal(submitted, 0, 'a funding failure must not reach submitMove');
});

test('an RPC failure while submitting is an error, not a promise that never settles', async () => {
  const model = await runMatch({
    config,
    previousState: previous,
    client: {
      submitMove: async () => { throw new Error('sendTransaction: timeout'); },
      readState: async () => afterMove,
    },
    policy: fastPolicy(),
  });

  assert.equal(model.phase, 'error');
  assert.equal(model.error.kind, 'rpc');
  assert.equal(model.error.detail, 'sendTransaction: timeout');
  assert.equal(model.txHash, null);
});

test('an RPC failure while polling is an error and keeps the last confirmed board', async () => {
  let reads = 0;

  const model = await runMatch({
    config,
    previousState: previous,
    client: {
      submitMove: async () => ({ txHash: 'tx-1' }),
      readState: async () => { reads += 1; throw new Error('get_state: connection reset'); },
    },
    policy: fastPolicy(),
  });

  assert.equal(model.phase, 'error');
  assert.equal(model.error.kind, 'rpc');
  assert.equal(model.error.detail, 'get_state: connection reset');
  assert.equal(model.state, previous, 'the board keeps the last confirmed state behind the error');
  assert.equal(model.txHash, 'tx-1');
  assert.equal(reads, 1);
});

test('a state that never changes times out after the bounded poll budget, so it cannot hang', async () => {
  let reads = 0;
  const frozen = { ...previous };

  const model = await runMatch({
    config,
    previousState: previous,
    client: {
      submitMove: async () => ({ txHash: 'tx-stuck' }),
      readState: async () => { reads += 1; return frozen; },
    },
    policy: fastPolicy({ finalityTimeoutMs: 3, pollIntervalMs: 1 }),
  });

  assert.equal(model.phase, 'error');
  assert.equal(model.error.kind, 'timeout');
  assert.equal(model.error.title, 'Timed out waiting for finality');
  assert.equal(reads, 3, 'the loop stops at maxPolls rather than polling forever');
  assert.equal(model.maxPolls, 3);
});

test('the finalizing phase carries the submitted hash and progress the UI can show', async () => {
  const seen = [];
  let reads = 0;

  await runMatch({
    config,
    previousState: previous,
    client: {
      submitMove: async () => ({ txHash: 'tx-progress' }),
      readState: async () => { reads += 1; return reads < 3 ? { ...previous } : afterMove; },
    },
    policy: fastPolicy({ finalityTimeoutMs: 5, pollIntervalMs: 1 }),
    hooks: {
      onPhase: (m) => {
        if (m.phase === 'finalizing' && m.txHash && m.polls > 0) seen.push({ polls: m.polls, txHash: m.txHash, state: m.state });
      },
    },
  });

  assert.equal(seen.length, 3, 'every poll during the wait is reported');
  assert.deepEqual(seen.map((entry) => entry.polls), [1, 2, 3]);
  for (const entry of seen) assert.equal(entry.txHash, 'tx-progress');
});

test('runMatch resolves rather than rejecting for non-Error throws', async () => {
  const model = await runMatch({
    config,
    previousState: previous,
    client: {
      submitMove: async () => { throw 'plain string failure'; },
      readState: async () => afterMove,
    },
    policy: fastPolicy(),
  });

  assert.equal(model.phase, 'error');
  assert.equal(model.error.detail, 'plain string failure');
});

test('runMatch rejects only for a malformed client, which is a programming error', async () => {
  await assert.rejects(runMatch({ config, client: {} }), /requires a client with submitMove and readState/);
});

test('describeRunError covers rpc, friendbot and timeout with actionable copy', () => {
  for (const kind of ['rpc', 'friendbot', 'timeout']) {
    const error = describeRunError(kind, { timeoutMs: 30000 });
    assert.equal(error.kind, kind);
    assert.ok(error.title.length > 0);
    assert.ok(error.message.length > 0);
    assert.ok(error.recovery.length > 0);
    assert.equal(error.retryable, true);
  }

  assert.match(describeRunError('timeout', { timeoutMs: 30000 }).message, /within 30s/);
  assert.match(describeRunError('friendbot').message, /No move was submitted/);
});

test('fixtures and live runs produce the same model shape', async () => {
  const fixture = loadFixture('3x3-win');
  const keyed = (model) => Object.keys(model).sort();

  const live = await runMatch({
    config,
    previousState: previous,
    client: { submitMove: async () => ({ txHash: 'tx' }), readState: async () => afterMove },
    policy: fastPolicy(),
  });

  const { modelFromFixture } = await import('../tools/fixtures.js');
  assert.deepEqual(keyed(modelFromFixture(fixture)), keyed(live));
});

test('DEFAULT_POLICY is a sane finality window for Stellar', () => {
  assert.equal(DEFAULT_POLICY.pollIntervalMs, 1500);
  assert.equal(DEFAULT_POLICY.finalityTimeoutMs, 30000);
  assert.ok(DEFAULT_POLICY.finalityTimeoutMs > DEFAULT_POLICY.pollIntervalMs);
});
