import test from 'node:test';
import assert from 'node:assert/strict';

import { defaultConfig } from '../src/config.js';
import {
  PHASES,
  TERMINAL_PHASES,
  filledCells,
  isTerminal,
  normalizeState,
  statusCaption,
  statusLabel,
  validateState,
} from '../src/state.js';
import { loadFixture } from '../tools/fixtures.js';

const config = defaultConfig('tic_tac_toe', 3, 3);

test('validateState accepts a contract-shaped state', () => {
  const fixture = loadFixture('3x3-win');
  assert.equal(validateState(fixture.config, fixture.state).ok, true);
});

test('validateState rejects a cell count that does not match the board', () => {
  const errors = validateState(config, { cells: [0, 0, 0], status: 0 }).errors;
  assert.equal(errors.length, 1);
  assert.match(errors[0], /3 cells but the board is 3×3 \(9\)/);
});

test('validateState rejects unknown status codes and out-of-range cell values', () => {
  const errors = validateState(config, { cells: [7, 0, 0, 0, 0, 0, 0, 0, 0], status: 9 }).errors;
  assert.deepEqual(errors, [
    'state.cells[0] must be 0, 1 or 2 (got 7)',
    'state.status 9 is not one of the config\'s status codes (0, 1, 2, 3)',
  ]);
});

test('normalizeState derives move_count when the contract omits it', () => {
  const normalized = normalizeState(config, { cells: [1, 0, 2, 0, 0, 0, 0, 0, 0], status: 0 });
  assert.equal(normalized.move_count, 2);
  assert.equal(normalized.is_x_turn, true);
});

test('statusLabel is authoritative for win, draw and in-progress', () => {
  assert.equal(statusLabel(loadFixture('3x3-win').config, loadFixture('3x3-win').state), 'X wins');
  assert.equal(statusLabel(loadFixture('3x3-draw').config, loadFixture('3x3-draw').state), 'Draw');
  assert.equal(statusLabel(config, { cells: new Array(9).fill(0), status: 0, is_x_turn: false, move_count: 0 }), 'O to move');
});

test('isTerminal only fires on win or draw', () => {
  const win = loadFixture('3x3-win');
  const draw = loadFixture('3x3-draw');
  const live = { cells: [1, 0, 0, 0, 2, 0, 0, 0, 0], status: 0, is_x_turn: true, move_count: 2 };

  assert.equal(isTerminal(win.config, win.state), true);
  assert.equal(isTerminal(draw.config, draw.state), true);
  assert.equal(isTerminal(config, live), false);
});

test('filledCells and statusCaption describe the board', () => {
  const fixture = loadFixture('3x3-win');
  assert.equal(filledCells(fixture.config, fixture.state), 5);
  assert.equal(statusCaption(fixture.config, fixture.state), '3×3 · move 5 · X wins');
});

test('PHASES and TERMINAL_PHASES stay in sync', () => {
  assert.deepEqual(TERMINAL_PHASES, [PHASES.SETTLED, PHASES.ERROR]);
});
