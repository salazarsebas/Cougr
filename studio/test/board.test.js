import test from 'node:test';
import assert from 'node:assert/strict';

import { BOARD_MAX_PX, boardGeometry, findWinningCells, renderBoard } from '../src/board.js';
import { defaultConfig } from '../src/config.js';
import { loadFixture } from '../tools/fixtures.js';

const count = (haystack, needle) => haystack.split(needle).length - 1;

test('geometry derives from the config, not from a fixed 3x3 canvas', () => {
  const three = boardGeometry(defaultConfig('t', 3, 3));
  assert.deepEqual(three, { cell: 160, cols: 3, rows: 3, widthPx: 480, heightPx: 480 });

  const connectFour = boardGeometry(defaultConfig('c', 7, 6, { winLength: 4 }));
  assert.equal(connectFour.cell, Math.floor(BOARD_MAX_PX / 7));
  assert.equal(connectFour.cols, 7);
  assert.equal(connectFour.rows, 6);
  assert.equal(connectFour.widthPx, connectFour.cell * 7);
  assert.equal(connectFour.heightPx, connectFour.cell * 6);
  assert.notEqual(connectFour.widthPx, connectFour.heightPx, 'a 7x6 board must not render square');
});

test('3x3 win fixture renders nine cells, five marks, and the winning line', () => {
  const fixture = loadFixture('3x3-win');
  const svg = renderBoard(fixture.config, fixture.state);

  assert.equal(count(svg, 'data-cell-index='), 9);
  assert.equal(count(svg, 'data-mark-label="X"'), 3);
  assert.equal(count(svg, 'data-mark-label="O"'), 2);
  assert.equal(count(svg, 'data-winning="true"'), 3);
  assert.match(svg, /viewBox="0 0 480 480"/);

  for (const index of fixture.expect.winningCells) {
    assert.match(svg, new RegExp(`data-cell-index="${index}"[^>]*data-winning="true"`));
  }
});

test('7x6 win fixture renders the full 42-cell board with a 4-in-a-row highlight', () => {
  const fixture = loadFixture('7x6-win');
  const svg = renderBoard(fixture.config, fixture.state);
  const geometry = boardGeometry(fixture.config);

  assert.equal(count(svg, 'data-cell-index='), 42);
  assert.match(svg, new RegExp(`viewBox="0 0 ${geometry.widthPx} ${geometry.heightPx}"`));
  assert.match(svg, /data-cols="7" data-rows="6"/);
  assert.equal(count(svg, 'data-winning="true"'), 4);
  assert.equal(count(svg, 'data-mark-label="R"'), 6);
  assert.equal(count(svg, 'data-mark-label="Y"'), 5);
});

test('draw fixture renders every cell but highlights none', () => {
  const fixture = loadFixture('3x3-draw');
  const svg = renderBoard(fixture.config, fixture.state);

  assert.equal(count(svg, 'data-cell-index='), 9);
  assert.equal(count(svg, 'data-mark-label='), 9);
  assert.equal(count(svg, 'data-winning="true"'), 0);
});

test('a null state renders the config board empty instead of failing', () => {
  const config = defaultConfig('t', 3, 3);
  const svg = renderBoard(config, null);

  assert.equal(count(svg, 'data-cell-index='), 9);
  assert.equal(count(svg, 'data-mark="0"'), 9);
  assert.equal(count(svg, 'data-mark-label='), 0);
});

test('findWinningCells scans rows, columns and both diagonals for winLength', () => {
  const config = defaultConfig('t', 3, 3);

  assert.deepEqual(findWinningCells(config, [1, 1, 1, 2, 2, 0, 0, 0, 0]), [0, 1, 2]);
  assert.deepEqual(findWinningCells(config, [2, 1, 0, 2, 1, 0, 2, 0, 1]), [0, 3, 6]);
  assert.deepEqual(findWinningCells(config, [1, 2, 2, 0, 1, 0, 0, 0, 1]), [0, 4, 8]);
  assert.deepEqual(findWinningCells(config, [0, 0, 2, 0, 2, 0, 2, 0, 0]), [2, 4, 6]);
  assert.deepEqual(findWinningCells(config, [1, 2, 1, 1, 2, 2, 2, 1, 1]), []);
});

test('findWinningCells honours a winLength that is not the board side', () => {
  const config = defaultConfig('connect_four', 7, 6, { winLength: 4 });
  const cells = new Array(42).fill(0);
  for (const index of [22, 23, 24, 25]) cells[index] = 1;

  assert.deepEqual(findWinningCells(config, cells), [22, 23, 24, 25]);

  // Three in a row is not a win at winLength 4.
  const three = new Array(42).fill(0);
  for (const index of [22, 23, 24]) three[index] = 1;
  assert.deepEqual(findWinningCells(config, three), []);
});
