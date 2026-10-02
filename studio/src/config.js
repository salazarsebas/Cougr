/**
 * `TurnBasedConfig` — the studio's client-side view of a turn-based match's
 * shape.
 *
 * The on-chain side of a turn-based Cougr contract stores its shape in Rust
 * (`GameState.cells` is a flat `Vec<u32>` plus a status code and a turn flag;
 * see `cli/templates/turn-based/src/lib.rs`). Nothing in the contract fixes the
 * board at 3×3 — the examples range from 3×3 tic-tac-toe to 7-wide Connect
 * Four to 9×9 sudoku — so the studio takes the shape as data instead of
 * hard-coding a canvas. `TurnBasedConfig` is that data.
 *
 * Shape:
 *
 *   {
 *     id:        string,                       // stable game id, e.g. "tic_tac_toe"
 *     label:     string,                       // human label for the header
 *     board:     { width, height },            // integer cell counts, row-major
 *     winLength: number,                       // consecutive marks that win
 *     marks:     { "1": string, "2": string }, // labels for cell values 1 and 2
 *     statusMap: {                             // GameState.status codes
 *       inProgress, firstWins, secondWins, draw
 *     }
 *   }
 *
 * Cell values are the contract's: `0` empty, `1` first player, `2` second
 * player. Anything else is a fixture bug and is rejected rather than guessed.
 */

/**
 * Upper bounds. These are not game rules — they keep a malformed fixture from
 * asking the DOM and the SVG renderer for millions of cells. Every legal Cougr
 * example is far below them.
 */
export const LIMITS = Object.freeze({
  minDimension: 1,
  maxDimension: 64,
  minWinLength: 1,
});

/** Status codes used by the turn-based contract template. */
export const DEFAULT_STATUS_MAP = Object.freeze({
  inProgress: 0,
  firstWins: 1,
  secondWins: 2,
  draw: 3,
});

const isInteger = (value) => Number.isInteger(value);
const isNonEmptyString = (value) => typeof value === 'string' && value.trim().length > 0;

/** Number of cells a config's board addresses. */
export function cellCount(config) {
  return config.board.width * config.board.height;
}

/** Row-major index for a cell, from zero-based row and column. */
export function cellIndex(config, row, column) {
  return row * config.board.width + column;
}

/** Row and column for a row-major index. */
export function cellPosition(config, index) {
  return { row: Math.floor(index / config.board.width), column: index % config.board.width };
}

/**
 * The label drawn for a cell value. Values outside `1` and `2` are empty cells
 * unless a caller deliberately asks for them, which keeps `0` from ever
 * rendering a stray glyph.
 */
export function markLabel(config, value) {
  if (value === 0 || value === null || value === undefined) return '';
  const label = config.marks[String(value)];
  return isNonEmptyString(label) ? label : `P${value}`;
}

/** The player to move, derived from `is_x_turn` (first player is the default). */
export function playerToMove(config, state) {
  return state.is_x_turn === false ? 2 : 1;
}

/**
 * Report every way `config` is not a legal `TurnBasedConfig`.
 *
 * Returns `{ ok, errors }` rather than throwing so fixture validation can
 * collect all problems at once instead of failing on the first one.
 */
export function validateConfig(config) {
  const errors = [];

  if (config === null || typeof config !== 'object') {
    return { ok: false, errors: ['config must be an object'] };
  }

  for (const key of ['id', 'label']) {
    if (!isNonEmptyString(config[key])) errors.push(`config.${key} must be a non-empty string`);
  }

  const board = config.board;
  if (board === null || typeof board !== 'object') {
    errors.push('config.board must be an object with width and height');
  } else {
    for (const axis of ['width', 'height']) {
      const value = board[axis];
      if (!isInteger(value)) {
        errors.push(`config.board.${axis} must be an integer`);
      } else if (value < LIMITS.minDimension || value > LIMITS.maxDimension) {
        errors.push(
          `config.board.${axis} must be between ${LIMITS.minDimension} and ${LIMITS.maxDimension} (got ${value})`,
        );
      }
    }
  }

  const legalDims = board && isInteger(board.width) && isInteger(board.height)
    && board.width >= LIMITS.minDimension && board.width <= LIMITS.maxDimension
    && board.height >= LIMITS.minDimension && board.height <= LIMITS.maxDimension;

  if (!isInteger(config.winLength)) {
    errors.push('config.winLength must be an integer');
  } else if (config.winLength < LIMITS.minWinLength) {
    errors.push(`config.winLength must be at least ${LIMITS.minWinLength}`);
  } else if (legalDims && config.winLength > Math.max(board.width, board.height)) {
    errors.push(
      `config.winLength (${config.winLength}) cannot exceed the board's longest side (${Math.max(board.width, board.height)})`,
    );
  }

  const marks = config.marks;
  if (marks === null || typeof marks !== 'object') {
    errors.push('config.marks must be an object mapping cell values to labels');
  } else {
    for (const value of ['1', '2']) {
      if (!isNonEmptyString(marks[value])) errors.push(`config.marks["${value}"] must be a non-empty string`);
    }
  }

  const statusMap = config.statusMap;
  if (statusMap === null || typeof statusMap !== 'object') {
    errors.push('config.statusMap must be an object of status codes');
  } else {
    const codes = [];
    for (const key of ['inProgress', 'firstWins', 'secondWins', 'draw']) {
      const value = statusMap[key];
      if (!isInteger(value) || value < 0) {
        errors.push(`config.statusMap.${key} must be a non-negative integer`);
      } else {
        codes.push(value);
      }
    }
    if (codes.length === 4 && new Set(codes).size !== 4) {
      errors.push('config.statusMap codes must be distinct so win and draw cannot be confused');
    }
  }

  return { ok: errors.length === 0, errors };
}

/** Throw a single error carrying every validation problem. */
export function assertLegalConfig(config) {
  const { ok, errors } = validateConfig(config);
  if (!ok) {
    throw new TypeError(`illegal TurnBasedConfig:\n  - ${errors.join('\n  - ')}`);
  }
  return config;
}

/**
 * Build a config with defaults, for a game id and board size. The `overrides`
 * object wins over the derived values, so a fixture only states what is
 * specific to its game.
 */
export function defaultConfig(id, width, height, overrides = {}) {
  return {
    id,
    label: id,
    board: { width, height },
    winLength: Math.min(width, height),
    marks: { 1: 'X', 2: 'O' },
    statusMap: { ...DEFAULT_STATUS_MAP },
    ...overrides,
  };
}
