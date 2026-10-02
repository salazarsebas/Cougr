import test from 'node:test';
import assert from 'node:assert/strict';

import {
  DEFAULT_STATUS_MAP,
  assertLegalConfig,
  cellCount,
  cellIndex,
  cellPosition,
  defaultConfig,
  markLabel,
  playerToMove,
  validateConfig,
} from '../src/config.js';

test('defaultConfig builds a legal 3x3 config', () => {
  const config = defaultConfig('tic_tac_toe', 3, 3);
  assert.equal(validateConfig(config).ok, true);
  assert.equal(config.winLength, 3);
  assert.equal(config.statusMap.firstWins, 1);
  assert.equal(cellCount(config), 9);
});

test('legal configs: any width and height, including non-square and larger boards', () => {
  for (const [width, height, winLength] of [[1, 1, 1], [3, 3, 3], [7, 6, 4], [9, 9, 5], [64, 64, 4]]) {
    const config = defaultConfig('game', width, height, { winLength });
    const result = validateConfig(config);
    assert.equal(result.ok, true, `${width}x${height} winLength ${winLength}: ${result.errors.join('; ')}`);
  }
});

test('rejects a board dimension that is not a positive integer inside the limit', () => {
  assert.deepEqual(
    validateConfig(defaultConfig('game', 0, 3, { winLength: 3 })).errors,
    ['config.board.width must be between 1 and 64 (got 0)'],
  );
  assert.match(validateConfig(defaultConfig('game', 3.5, 3)).errors[0], /must be an integer/);
  assert.match(validateConfig(defaultConfig('game', 65, 3)).errors[0], /between 1 and 64/);
});

test('rejects a winLength longer than the board has room for', () => {
  const errors = validateConfig(defaultConfig('game', 3, 3, { winLength: 5 })).errors;
  assert.equal(errors.length, 1);
  assert.match(errors[0], /winLength \(5\) cannot exceed the board's longest side \(3\)/);
});

test('rejects a winLength below one', () => {
  assert.match(validateConfig(defaultConfig('game', 3, 3, { winLength: 0 })).errors[0], /at least 1/);
});

test('rejects duplicate status codes, so a win cannot be read as a draw', () => {
  const config = defaultConfig('game', 3, 3, { statusMap: { ...DEFAULT_STATUS_MAP, draw: 1 } });
  assert.match(validateConfig(config).errors[0], /must be distinct/);
});

test('rejects empty marks', () => {
  const config = defaultConfig('game', 3, 3, { marks: { 1: 'X', 2: ' ' } });
  assert.match(validateConfig(config).errors[0], /config\.marks\["2"\] must be a non-empty string/);
});

test('assertLegalConfig names every problem at once', () => {
  assert.throws(
    () => assertLegalConfig({ id: '', label: '', board: { width: 0, height: -1 }, winLength: 0, marks: {}, statusMap: {} }),
    (error) => {
      assert.ok(error instanceof TypeError);
      assert.equal(error.message.split('\n').length > 5, true);
      return true;
    },
  );
});

test('cellIndex and cellPosition round-trip on a non-square board', () => {
  const config = defaultConfig('game', 7, 6, { winLength: 4 });
  assert.equal(cellIndex(config, 3, 4), 25);
  assert.deepEqual(cellPosition(config, 25), { row: 3, column: 4 });
  assert.deepEqual(cellPosition(config, 41), { row: 5, column: 6 });
});

test('markLabel renders empty for 0 and falls back for unknown values', () => {
  const config = defaultConfig('game', 3, 3);
  assert.equal(markLabel(config, 0), '');
  assert.equal(markLabel(config, 1), 'X');
  assert.equal(markLabel(config, 2), 'O');
  assert.equal(markLabel(config, 3), 'P3');
});

test('playerToMove defaults to the first player and honours is_x_turn', () => {
  const config = defaultConfig('game', 3, 3);
  assert.equal(playerToMove(config, { is_x_turn: true }), 1);
  assert.equal(playerToMove(config, { is_x_turn: false }), 2);
  assert.equal(playerToMove(config, {}), 1);
});
