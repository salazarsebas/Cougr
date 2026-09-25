#!/usr/bin/env node
/**
 * Validate every recorded fixture against the studio's contracts.
 *
 * Two things are checked, and both are the reason this runs in CI rather than
 * being trusted:
 *
 *   1. Structural legality — the config is a legal `TurnBasedConfig` and the
 *      state matches it (cell count, status code, cell values). A fixture that
 *      breaks the contract fails here, not three layers down in the renderer.
 *   2. Declared expectations — the fixture's `expect` block must agree with
 *      what the board and status actually compute. This is what stops a
 *      fixture from drifting silently: if a renderer change moves a cell or a
 *      label, the fixture stops matching and CI says so.
 *
 * Usage: node tools/validate-fixtures.js
 */

import { assertLegalConfig, cellCount } from '../src/config.js';
import { findWinningCells } from '../src/board.js';
import { assertLegalState, filledCells, isTerminal, statusLabel } from '../src/state.js';
import { listFixtures, loadFixture, modelFromFixture } from './fixtures.js';

/** Compare the computed values against whatever the fixture declared. */
function checkExpectations(fixture) {
  const problems = [];
  const { config, state, expect = {}, run = {} } = fixture;

  if (state) {
    assertLegalState(config, state);

    const computed = {
      rows: config.board.height,
      cols: config.board.width,
      cellCount: cellCount(config),
      filledCells: filledCells(config, state),
      statusLabel: statusLabel(config, state),
      winningCells: isTerminal(config, state) ? findWinningCells(config, state.cells) : [],
      terminal: isTerminal(config, state),
    };

    for (const [key, actual] of Object.entries(computed)) {
      if (!(key in expect)) continue;
      const declared = expect[key];
      const same = Array.isArray(actual)
        ? JSON.stringify(actual) === JSON.stringify(declared)
        : actual === declared;
      if (!same) {
        problems.push(`expect.${key}: declared ${JSON.stringify(declared)}, computed ${JSON.stringify(actual)}`);
      }
    }
  }

  if ('phase' in expect && expect.phase !== (run.phase ?? 'idle')) {
    problems.push(`expect.phase: declared ${JSON.stringify(expect.phase)}, fixture run phase ${JSON.stringify(run.phase ?? 'idle')}`);
  }

  if ('errorKind' in expect && expect.errorKind !== (run.errorKind ?? null)) {
    problems.push(`expect.errorKind: declared ${JSON.stringify(expect.errorKind)}, fixture run errorKind ${JSON.stringify(run.errorKind ?? null)}`);
  }

  return problems;
}

function main() {
  const ids = listFixtures();
  const failures = [];

  for (const id of ids) {
    const fixture = loadFixture(id);
    const problems = [];

    try {
      assertLegalConfig(fixture.config);
      problems.push(...checkExpectations(fixture));
    } catch (error) {
      problems.push(error.message);
    }

    if (problems.length > 0) failures.push({ id, problems });

    const model = problems.length === 0 ? modelFromFixture(fixture) : null;
    const board = fixture.config?.board;
    const summary = board ? `${board.width}×${board.height}` : 'no board';
    const cellTotal = fixture.config && board ? cellCount(fixture.config) : 0;
    const status = problems.length === 0 ? statusLabel(fixture.config, fixture.state) : 'invalid';
    const phase = model ? model.phase : 'invalid';

    console.log(
      `${problems.length === 0 ? '✓' : '✗'} ${id.padEnd(22)} ${summary.padEnd(6)} ${String(cellTotal).padStart(3)} cells  ${phase.padEnd(10)} ${status}`,
    );
  }

  console.log(`\n${ids.length} fixtures, ${failures.length} failing`);

  for (const { id, problems } of failures) {
    console.error(`\n${id}:`);
    for (const problem of problems) console.error(`  - ${problem}`);
  }

  if (failures.length > 0) process.exit(1);
}

main();
