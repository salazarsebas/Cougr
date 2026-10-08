/**
 * Turn-based game state helpers.
 *
 * Decodes the binary formats emitted by the `turn-based` template's
 * `impl_component!(TurnState, ...)` and reconstructs `GameState` from a
 * decoded `sdk-events` update — without a follow-up `get_state` RPC call.
 */

import type { ComponentSetUpdate, CougrEventUpdate } from 'cougr-sdk-events';

// ── Domain types ──────────────────────────────────────────────────────────

/**
 * The on-chain game state for a tic-tac-toe turn-based game.
 *
 * Will align with the `GameState` type exported by `cougr-sdk-core` once
 * PR #350 lands.  Until then these types are standalone here and in stubs.ts.
 */
export interface GameState {
  cells: number[];        // 9 cells, 0=empty 1=X 2=O
  player_x: string;       // Stellar address
  player_o: string;       // Stellar address
  is_x_turn: boolean;
  move_count: number;
  status: number;         // 0=in_progress 1=x_wins 2=o_wins 3=draw
}

/**
 * The return value of a `make_move` contract invocation.
 *
 * Will align with the `MoveResult` type exported by `cougr-sdk-core` once
 * PR #350 lands.
 */
export interface MoveResult {
  success: boolean;
  game_state: GameState;
  message: string;        // "ok" | "gameover" | "bounds" | "occupied" | "notturn" | "notplay"
}

// ── Component names (must match Rust impl_component! / impl_rich_component!) ─

export const TURN_STATE_COMPONENT = 'turnst';
export const BOARD_COMPONENT = 'board';
export const PLAYERS_COMPONENT = 'players';

// ── Status codes ──────────────────────────────────────────────────────────

export const STATUS_IN_PROGRESS = 0;
export const STATUS_X_WINS = 1;
export const STATUS_O_WINS = 2;
export const STATUS_DRAW = 3;

export type TurnStatus = 'in_progress' | 'x_wins' | 'o_wins' | 'draw' | 'unknown';

export function statusLabel(status: number): TurnStatus {
  switch (status) {
    case STATUS_IN_PROGRESS: return 'in_progress';
    case STATUS_X_WINS: return 'x_wins';
    case STATUS_O_WINS: return 'o_wins';
    case STATUS_DRAW: return 'draw';
    default: return 'unknown';
  }
}

// ── TurnState binary decoder ──────────────────────────────────────────────

export interface DecodedTurnState {
  isXTurn: boolean;
  moveCount: number;
  /** One of STATUS_* constants. */
  status: number;
  statusLabel: TurnStatus;
}

/**
 * Decode the `data` bytes from a `("COUGR", "set", "turnst")` event.
 *
 * Wire layout from:
 *   `impl_component!(TurnState, "turnst", Table, { is_x_turn: bool, move_count: u32, status: u32 })`
 *
 * - byte 0:    `bool`  — `is_x_turn` (1=true, 0=false)
 * - bytes 1–4: `u32` big-endian — `move_count`
 * - bytes 5–8: `u32` big-endian — `status`
 */
export function decodeTurnState(data: Uint8Array): DecodedTurnState {
  if (data.length < 9) {
    throw new TypeError(
      `TurnState data must be at least 9 bytes, got ${data.length}`,
    );
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const isXTurn = (data[0] ?? 0) !== 0;
  const moveCount = view.getUint32(1, false /* big-endian */);
  const status = view.getUint32(5, false);
  return { isXTurn, moveCount, status, statusLabel: statusLabel(status) };
}

// ── GameState reconstruction from events ──────────────────────────────────

/**
 * Apply a `turnst` set-update to an existing `GameState` snapshot, returning
 * an updated snapshot **without** making a follow-up `get_state` RPC call.
 *
 * This is the core claim of issue #367: after `make_move` the client learns
 * the new turn position entirely from the decoded `sdk-events` update.
 *
 * Components that changed:
 *   - `TurnState` emits a `("COUGR", "set", "turnst")` event → binary decode
 *   - `Board` emits a `("COUGR", "rich", "board")` event → follow-up read
 *     required by the sdk-events type system (`requiresFollowUpRead: true`)
 *
 * For `TurnState` we get the full new value in the event. For `Board` (rich
 * component) the new cell layout must be obtained via a follow-up read; until
 * that is done we keep the previous board. A real client would call
 * `resolveRichComponentUpdate` on the Board update.
 */
export function applyTurnStateUpdate(
  prev: GameState,
  update: ComponentSetUpdate,
): GameState {
  const decoded = decodeTurnState(update.data);
  return {
    ...prev,
    is_x_turn: decoded.isXTurn,
    move_count: decoded.moveCount,
    status: decoded.status,
  };
}

/**
 * Scan a list of decoded updates from a single ledger and apply any
 * `TurnState` set-updates to `prev`, returning the latest known game state.
 *
 * Board and Players rich-component updates require separate follow-up reads
 * (the `requiresFollowUpRead: true` on `RichComponentChangedUpdate` enforces
 * this at the type level).
 */
export function applyUpdatesToGameState(
  prev: GameState,
  updates: readonly CougrEventUpdate[],
): { state: GameState; richPending: readonly CougrEventUpdate[] } {
  let state = prev;
  const richPending: CougrEventUpdate[] = [];

  for (const update of updates) {
    if (update.family === 'set' && update.componentType === TURN_STATE_COMPONENT) {
      state = applyTurnStateUpdate(state, update);
    } else if (update.family === 'rich') {
      // Rich-component change: cannot be applied inline.
      // The caller must call resolveRichComponentUpdate (from sdk-events) to
      // fetch the new value and merge it separately.
      richPending.push(update);
    }
    // 'del' events on turn-state are not expected in normal gameplay.
  }

  return { state, richPending };
}

// ── ASCII board renderer ──────────────────────────────────────────────────

export function formatCell(cell: number): string {
  return cell === 1 ? 'X' : cell === 2 ? 'O' : '.';
}

export function renderBoard(cells: readonly number[], width = 3, height = 3): string {
  const rows: string[] = [];
  for (let r = 0; r < height; r += 1) {
    const row: string[] = [];
    for (let c = 0; c < width; c += 1) {
      row.push(formatCell(cells[r * width + c] ?? 0));
    }
    rows.push(' ' + row.join(' | ') + ' ');
  }
  const divider = '\n---+---+---\n';
  return rows.join(divider);
}
