import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  authorizeSession,
  deriveSessionKeyId,
  toHex,
  type LedgerClock,
  type SessionKey,
  type SessionScope,
} from '../src/index.ts';

interface RawScope {
  allowedActions: string[];
  maxOperations: number;
  expiresAt: number;
}

interface ActionVector {
  scope: RawScope;
  key: { createdAt: number; operationsUsed: number; nextNonce: number };
  now: number;
  action: string;
  nonce: number;
  expected: {
    accepted: boolean;
    errorName?: string;
    errorCode?: number;
    remainingOperations?: number;
    operationsUsed?: number;
    nextNonce?: number;
  };
}

interface Vectors {
  keyId: {
    inputs: {
      timestamp: number;
      sequence: number;
      existingSessions: number;
      scope: RawScope;
    };
    expectedHex: string;
  };
  validSession: ActionVector;
  rejectedReplay: ActionVector;
  expiredSession: ActionVector;
  actionNotAllowed: ActionVector;
  budgetExceeded: ActionVector;
}

const vectors = JSON.parse(
  readFileSync(new URL('../vectors/session-vectors.json', import.meta.url), 'utf8'),
) as Vectors;

function clockAt(timestamp: number, sequence = 0): LedgerClock {
  return { timestamp: () => BigInt(timestamp), sequence: () => sequence };
}

function toScope(raw: RawScope): SessionScope {
  return {
    allowedActions: raw.allowedActions,
    maxOperations: raw.maxOperations,
    expiresAt: BigInt(raw.expiresAt),
  };
}

function keyOf(vector: ActionVector): SessionKey {
  const keyId = new Uint8Array(32);
  keyId.fill(7);
  return {
    keyId,
    scope: toScope(vector.scope),
    createdAt: BigInt(vector.key.createdAt),
    operationsUsed: vector.key.operationsUsed,
    nextNonce: BigInt(vector.key.nextNonce),
  };
}

test('keyId vector matches the Rust derivation byte for byte', () => {
  const { inputs, expectedHex } = vectors.keyId;
  const derived = deriveSessionKeyId({
    timestamp: BigInt(inputs.timestamp),
    sequence: inputs.sequence,
    existingSessions: inputs.existingSessions,
    scope: toScope(inputs.scope),
  });

  assert.equal(derived.length, 32);
  assert.equal(toHex(derived), expectedHex);
});

test('valid session is accepted and advances nonce and budget', () => {
  const vector = vectors.validSession;
  const result = authorizeSession(
    keyOf(vector),
    vector.action,
    BigInt(vector.nonce),
    clockAt(vector.now),
  );

  if (!result.ok) {
    assert.fail(`expected acceptance, got ${result.error.message}`);
  }
  assert.equal(result.remainingOperations, vector.expected.remainingOperations);
  assert.equal(result.key.operationsUsed, vector.expected.operationsUsed);
  assert.equal(result.key.nextNonce, BigInt(vector.expected.nextNonce));
});

test('replayed nonce is rejected with the contract error code', () => {
  const vector = vectors.rejectedReplay;
  const result = authorizeSession(
    keyOf(vector),
    vector.action,
    BigInt(vector.nonce),
    clockAt(vector.now),
  );

  if (result.ok) {
    assert.fail('expected the replayed nonce to be rejected');
  }
  assert.equal(result.error.sessionError, vector.expected.errorName);
  assert.equal(result.error.code, vector.expected.errorCode);
});

test('expired session is rejected before any submit', () => {
  const vector = vectors.expiredSession;
  const result = authorizeSession(
    keyOf(vector),
    vector.action,
    BigInt(vector.nonce),
    clockAt(vector.now),
  );

  if (result.ok) {
    assert.fail('expected the expired session to be rejected');
  }
  assert.equal(result.error.sessionError, vector.expected.errorName);
  assert.equal(result.error.code, vector.expected.errorCode);
});

test('action outside the scope is rejected', () => {
  const vector = vectors.actionNotAllowed;
  const result = authorizeSession(
    keyOf(vector),
    vector.action,
    BigInt(vector.nonce),
    clockAt(vector.now),
  );

  if (result.ok) {
    assert.fail('expected the out-of-scope action to be rejected');
  }
  assert.equal(result.error.sessionError, vector.expected.errorName);
  assert.equal(result.error.code, vector.expected.errorCode);
});

test('exhausted budget is rejected', () => {
  const vector = vectors.budgetExceeded;
  const result = authorizeSession(
    keyOf(vector),
    vector.action,
    BigInt(vector.nonce),
    clockAt(vector.now),
  );

  if (result.ok) {
    assert.fail('expected the exhausted budget to be rejected');
  }
  assert.equal(result.error.sessionError, vector.expected.errorName);
  assert.equal(result.error.code, vector.expected.errorCode);
});
