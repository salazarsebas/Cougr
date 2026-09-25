import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  SessionBuilder,
  SessionClient,
  SessionError,
  authorizeSession,
  isExpired,
  validateSession,
  type LedgerClock,
  type SessionKey,
  type SessionKeyProvider,
} from '../src/index.ts';

const CLOCK: LedgerClock = { timestamp: () => 1_735_689_700n, sequence: () => 42 };

function keyWith(overrides: Partial<SessionKey> = {}): SessionKey {
  return {
    keyId: new Uint8Array(32),
    scope: { allowedActions: ['move', 'attack'], maxOperations: 2, expiresAt: 1_735_693_200n },
    createdAt: 1_735_689_600n,
    operationsUsed: 0,
    nextNonce: 0n,
    ...overrides,
  };
}

test('builder mirrors SessionBuilder scope construction', () => {
  const scope = new SessionBuilder(CLOCK)
    .allowAction('move')
    .allowAction('attack')
    .maxOperations(2)
    .expiresAt(1_735_693_200n)
    .buildScope();

  assert.deepEqual(scope.allowedActions, ['move', 'attack']);
  assert.equal(scope.maxOperations, 2);
  assert.equal(scope.expiresAt, 1_735_693_200n);
});

test('expiresIn resolves against the ledger clock like the Rust builder', () => {
  const scope = new SessionBuilder(CLOCK).expiresIn(800n).buildScope();
  assert.equal(scope.expiresAt, 1_735_690_500n);
});

test('builder creates a key through the provider interface', async () => {
  const provider: SessionKeyProvider = {
    createSession: (scope) => keyWith({ scope }),
  };
  const key = await new SessionBuilder(CLOCK).allowAction('move').maxOperations(1).create(provider);

  assert.deepEqual(key.scope.allowedActions, ['move']);
  assert.equal(key.scope.maxOperations, 1);
});

test('expiry boundary matches the contract (now >= expiresAt is expired)', () => {
  const key = keyWith();
  assert.equal(isExpired(key, CLOCK), false);
  assert.equal(validateSession(key, CLOCK), true);

  const atBoundary: LedgerClock = { timestamp: () => 1_735_693_200n, sequence: () => 42 };
  assert.equal(isExpired(key, atBoundary), true);
  assert.equal(validateSession(key, atBoundary), false);
});

test('authorize consumes budget and nonce exactly like the contract', () => {
  const result = authorizeSession(keyWith(), 'move', 0n, CLOCK);

  if (!result.ok) {
    assert.fail(`expected acceptance, got ${result.error.message}`);
  }
  assert.equal(result.remainingOperations, 1);
  assert.equal(result.key.operationsUsed, 1);
  assert.equal(result.key.nextNonce, 1n);
});

test('client rejects a stale nonce before submit and stays unchanged', () => {
  const client = new SessionClient(keyWith({ operationsUsed: 1, nextNonce: 1n }), CLOCK);

  assert.throws(() => client.assertFreshNonce(0n), SessionError);
  assert.equal(client.key.operationsUsed, 1);

  const result = client.authorize('move', 1n);
  if (!result.ok) {
    assert.fail(`expected acceptance, got ${result.error.message}`);
  }
  assert.equal(client.key.nextNonce, 2n);
  assert.equal(client.key.operationsUsed, 2);
});

test('client refuses an expired session before building a transaction', () => {
  const expired: LedgerClock = { timestamp: () => 1_735_694_000n, sequence: () => 42 };
  const client = new SessionClient(keyWith(), expired);

  assert.equal(client.validate(), false);
  const result = client.authorize('move', 0n);
  if (result.ok) {
    assert.fail('expected the expired session to be rejected');
  }
  assert.equal(result.error.code, 21);
});

test('out-of-scope action is rejected after expiry and nonce pass', () => {
  const result = authorizeSession(keyWith(), 'dance', 0n, CLOCK);
  if (result.ok) {
    assert.fail('expected the out-of-scope action to be rejected');
  }
  assert.equal(result.error.code, 38);
});

test('invalid action symbols are rejected at build time', () => {
  assert.throws(() => new SessionBuilder(CLOCK).allowAction('Move!'), SessionError);
});
