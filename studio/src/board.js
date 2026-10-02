/**
 * Board renderer — draws any legal `TurnBasedConfig`.
 *
 * The board size comes from the config, so nothing here assumes 3×3. Geometry
 * is computed once per config: the longest side is scaled to `BOARD_MAX_PX`
 * with an integer cell size, which keeps every grid line and every mark on a
 * whole pixel and keeps a 7×6 board the same overall footprint as a 3×3.
 *
 * Output is a standalone SVG string: no DOM, no framework, and therefore
 * directly assertable from `node:test` and writable to disk as an artifact.
 */

import { cellCount, cellIndex, markLabel } from './config.js';
import { BRAND, FONT_STYLE, LINE, STUDIO, TINT, px } from './theme.js';

/** Longest side of the rendered board, in SVG user units. */
export const BOARD_MAX_PX = 480;

/** Grid stroke width, in SVG user units. */
const GRID_STROKE = 2;

/** Direction vectors used by the win scan: →, ↓, ↘, ↙. */
const DIRECTIONS = Object.freeze([
  [0, 1],
  [1, 0],
  [1, 1],
  [1, -1],
]);

/**
 * Cell size and board pixel size for a config.
 *
 * `cell` is floored so cells stay integral at any board size; the board is then
 * `cell × cols` by `cell × rows`, which is never larger than `BOARD_MAX_PX` on
 * its longest side and may be slightly smaller when the size does not divide
 * evenly.
 */
export function boardGeometry(config) {
  const { width, height } = config.board;
  const cell = Math.floor(BOARD_MAX_PX / Math.max(width, height));
  return {
    cell,
    cols: width,
    rows: height,
    widthPx: cell * width,
    heightPx: cell * height,
  };
}

/**
 * Indices of every cell that belongs to a completed line, or `[]`.
 *
 * This is a generic line scan, not a tic-tac-toe rule: it looks for
 * `winLength` consecutive cells holding the same non-empty value along rows,
 * columns and both diagonals. Games whose win condition is not a line (checkers
 * capture counts, for example) simply come back empty, and the renderer falls
 * back to the contract's `status` alone — which is why the highlight is
 * decorative and the status line is authoritative.
 */
export function findWinningCells(config, cells) {
  const { width: cols, height: rows } = config.board;
  const { winLength } = config;
  const winning = new Set();

  for (let row = 0; row < rows; row += 1) {
    for (let column = 0; column < cols; column += 1) {
      const value = cells[cellIndex(config, row, column)];
      if (!value) continue;

      for (const [dRow, dColumn] of DIRECTIONS) {
        const run = [cellIndex(config, row, column)];
        let matched = true;

        for (let step = 1; step < winLength; step += 1) {
          const nextRow = row + dRow * step;
          const nextColumn = column + dColumn * step;
          if (nextRow < 0 || nextRow >= rows || nextColumn < 0 || nextColumn >= cols) {
            matched = false;
            break;
          }
          if (cells[cellIndex(config, nextRow, nextColumn)] !== value) {
            matched = false;
            break;
          }
          run.push(cellIndex(config, nextRow, nextColumn));
        }

        if (matched) for (const index of run) winning.add(index);
      }
    }
  }

  return [...winning].sort((a, b) => a - b);
}

/**
 * Render the board as an SVG string.
 *
 * `state` may be `null` (before the first confirmed read) — the empty config
 * board is drawn with no marks, which is the correct thing to show while the
 * first read is still in flight.
 */
export function renderBoard(config, state, options = {}) {
  const { cell, cols, rows, widthPx, heightPx } = boardGeometry(config);
  const cells = state ? state.cells : new Array(cellCount(config)).fill(0);
  const showWinningCells = options.highlightWinning !== false && config.statusMap && state
    ? state.status !== config.statusMap.inProgress
    : false;
  const winning = showWinningCells ? new Set(findWinningCells(config, cells)) : new Set();
  const markSize = Math.round(cell * 0.44);

  const parts = [];
  parts.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${widthPx} ${heightPx}" width="${widthPx}" height="${heightPx}" role="img" aria-label="${escapeAttribute(boardLabel(config))}" data-board="${escapeAttribute(config.id)}" data-cols="${cols}" data-rows="${rows}">`);
  parts.push(`<title>${escapeText(boardLabel(config))}</title>`);
  parts.push(`<desc>${escapeText(`${cols} by ${rows} board, ${cellCount(config)} cells`)}</desc>`);
  parts.push(`<defs><style>${FONT_STYLE}</style></defs>`);
  parts.push(`<rect x="0" y="0" width="${widthPx}" height="${heightPx}" fill="${BRAND.colorBg}" rx="${px(BRAND.radiusLg)}"/>`);

  for (let row = 0; row < rows; row += 1) {
    for (let column = 0; column < cols; column += 1) {
      const index = cellIndex(config, row, column);
      const value = cells[index];
      const x = column * cell;
      const y = row * cell;
      const isWinning = winning.has(index);
      const fill = isWinning ? `${STUDIO.settled}${TINT}` : BRAND.colorSurface;

      parts.push(`<rect data-cell-index="${index}" data-row="${row}" data-col="${column}" data-mark="${value}"${isWinning ? ' data-winning="true"' : ''} x="${x + 1}" y="${y + 1}" width="${cell - GRID_STROKE}" height="${cell - GRID_STROKE}" rx="${px(BRAND.radiusSm)}" fill="${fill}" stroke="${LINE.stroke}" stroke-opacity="${LINE.gridOpacity}" stroke-width="${GRID_STROKE}"/>`);

      const label = markLabel(config, value);
      if (label) {
        const color = isWinning ? STUDIO.settled : value === 1 ? BRAND.colorPrimary : BRAND.colorAccent;
        parts.push(`<text data-mark-label="${escapeAttribute(label)}" x="${x + cell / 2}" y="${y + cell / 2}" text-anchor="middle" dominant-baseline="central" font-size="${markSize}" font-weight="700" fill="${color}">${escapeText(label)}</text>`);
      }
    }
  }

  parts.push('</svg>');
  return parts.join('');
}

function boardLabel(config) {
  return `${config.label} board — ${config.board.width} by ${config.board.height}`;
}

function escapeText(value) {
  return String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function escapeAttribute(value) {
  return escapeText(value).replace(/"/g, '&quot;');
}
