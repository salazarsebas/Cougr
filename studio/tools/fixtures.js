#!/usr/bin/env node
/**
 * Recorded-fixture loader.
 *
 * Node-only on purpose: the view modules in `src/` are host-agnostic and must
 * not import `node:fs`, so fixture loading lives here, next to the CLI tools
 * that need it. Tests and the renderer share this loader so a fixture is
 * parsed and normalized exactly once, the same way in both places.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { createRunModel, describeRunError } from '../src/run.js';

const HERE = path.dirname(fileURLToPath(import.meta.url));

/** Repository-relative root of the studio tree. */
export const STUDIO_DIR = path.resolve(HERE, '..');

/** Directory holding the recorded JSON fixtures. */
export const FIXTURES_DIR = path.join(STUDIO_DIR, 'fixtures');

/** Fixture ids, in filesystem order, without the `.json` suffix. */
export function listFixtures() {
  return fs
    .readdirSync(FIXTURES_DIR)
    .filter((file) => file.endsWith('.json'))
    .sort()
    .map((file) => file.slice(0, -'.json'.length));
}

/** Read one fixture by id. Throws with the available ids when it is missing. */
export function loadFixture(id) {
  const file = path.join(FIXTURES_DIR, `${id}.json`);
  if (!fs.existsSync(file)) {
    throw new Error(`unknown fixture "${id}" (available: ${listFixtures().join(', ')})`);
  }
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

/** Read every fixture, in id order. */
export function loadAllFixtures() {
  return listFixtures().map(loadFixture);
}

/**
 * Turn a recorded fixture into the same model `runMatch` produces, so the view
 * cannot tell the two apart — which is the point: fixtures are the backend
 * that does not exist yet, and live state is the same object.
 */
export function modelFromFixture(fixture) {
  const run = fixture.run ?? {};
  const previousState = fixture.previousState ?? fixture.state ?? null;

  return createRunModel({
    config: fixture.config,
    state: fixture.state ?? null,
    previousState,
    move: fixture.move ?? null,
    txHash: run.txHash ?? null,
    phase: run.phase ?? 'idle',
    elapsedMs: run.elapsedMs ?? 0,
    polls: run.polls ?? 0,
    maxPolls: run.maxPolls ?? null,
    error: run.errorKind
      ? describeRunError(run.errorKind, { detail: run.detail ?? null, timeoutMs: run.timeoutMs })
      : null,
  });
}
