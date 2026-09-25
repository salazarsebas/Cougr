/**
 * Match state and run lifecycle.
 *
 * Two shapes live here:
 *
 *   MatchState — the contract's `GameState`, unchanged:
 *     { cells: number[], status: number, is_x_turn: boolean, move_count: number }
 *
 *   RunPhase — what the studio is doing right now:
 *     idle -> funding -> submitting -> finalizing -> settled
 *                                              \-> error
 *
 * Keeping them separate is what lets the board keep drawing the last confirmed
 * state while the run panel explains the wait. The board never has to render a
 * spinner, and the spinner never has to know about cells.
 */

import { cellCount, markLabel, playerToMove } from './config.js';

/** Run lifecycle phases. */
export const PHASES = Object.freeze({
  IDLE: 'idle',
  FUNDING: 'funding',
  SUBMITTING: 'submitting',
  FINALIZING: 'finalizing',
  SETTLED: 'settled',
  ERROR: 'error',
});

/** Terminal run phases: no timer or poll may be outstanding once here. */
export const TERMINAL_PHASES = Object.freeze([PHASES.SETTLED, PHASES.ERROR]);

/**
 * Validate a match state against its config.
 *
 * `move_count` is optional on input — a contract that omits it has it derived
 * from the board — but `cells` and `status` are the state, so they are
 * required.
 */
export function validateState(config, state) {
  const errors = [];

  if (state === null || typeof state !== 'object') {
    return { ok: false, errors: ['state must be an object'] };
  }

  if (!Array.isArray(state.cells)) {
    errors.push('state.cells must be an array');
  } else {
    const expected = cellCount(config);
    if (state.cells.length !== expected) {
      errors.push(`state.cells has ${state.cells.length} cells but the board is ${config.board.width}×${config.board.height} (${expected})`);
    }
    state.cells.forEach((value, index) => {
      if (!Number.isInteger(value) || value < 0 || value > 2) {
        errors.push(`state.cells[${index}] must be 0, 1 or 2 (got ${JSON.stringify(value)})`);
      }
    });
  }

  const codes = config.statusMap ? Object.values(config.statusMap) : [];
  if (!Number.isInteger(state.status)) {
    errors.push('state.status must be an integer');
  } else if (codes.length > 0 && !codes.includes(state.status)) {
    errors.push(`state.status ${state.status} is not one of the config's status codes (${codes.join(', ')})`);
  }

  if (state.is_x_turn !== undefined && typeof state.is_x_turn !== 'boolean') {
    errors.push('state.is_x_turn must be a boolean when present');
  }

  if (state.move_count !== undefined && (!Number.isInteger(state.move_count) || state.move_count < 0)) {
    errors.push('state.move_count must be a non-negative integer when present');
  }

  return { ok: errors.length === 0, errors };
}

/** Throw a single error carrying every state problem. */
export function assertLegalState(config, state) {
  const { ok, errors } = validateState(config, state);
  if (!ok) {
    throw new TypeError(`state does not match the config:\n  - ${errors.join('\n  - ')}`);
  }
  return state;
}

/** Fill in the fields a contract may omit, without inventing board content. */
export function normalizeState(config, state) {
  const cells = state.cells.slice();
  const moveCount = state.move_count === undefined
    ? cells.filter((value) => value !== 0).length
    : state.move_count;

  return {
    cells,
    status: state.status,
    is_x_turn: state.is_x_turn === undefined ? true : state.is_x_turn,
    move_count: moveCount,
  };
}

/** Filled-cell count, useful for fixtures and the status line. */
export function filledCells(config, state) {
  return state.cells.filter((value) => value !== 0).length;
}

/** Whether the match has ended (win or draw), per the config's status codes. */
export function isTerminal(config, state) {
  return state.status !== config.statusMap.inProgress;
}

/** A short human status: "X wins", "Draw", or "<mark> to move". */
export function statusLabel(config, state) {
  const { statusMap } = config;
  if (state.status === statusMap.firstWins) return `${markLabel(config, 1)} wins`;
  if (state.status === statusMap.secondWins) return `${markLabel(config, 2)} wins`;
  if (state.status === statusMap.draw) return 'Draw';
  return `${markLabel(config, playerToMove(config, state))} to move`;
}

/** One-line caption for the board header. */
export function statusCaption(config, state) {
  return `${config.board.width}×${config.board.height} · move ${state.move_count} · ${statusLabel(config, state)}`;
}
