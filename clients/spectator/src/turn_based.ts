/**
 * Turn-based game domain helpers and decoders.
 *
 * Models the components emitted by Cougr's `turn-based` template and
 * `examples/tic_tac_toe`:
 * - `TurnState` (`"turnst"`): fixed-size 9-byte component (`is_x_turn: bool`, `move_count: u32`, `status: u32`)
 * - `Board` (`"board"`): rich or typed component containing cell markers (0=empty, 1=X, 2=O)
 * - `Players` (`"players"`): rich component containing player addresses
 */

export const TURN_STATE_COMPONENT = 'turnst';
export const BOARD_COMPONENT = 'board';
export const PLAYERS_COMPONENT = 'players';

export const STATUS_IN_PROGRESS = 0;
export const STATUS_X_WINS = 1;
export const STATUS_O_WINS = 2;
export const STATUS_DRAW = 3;

export const CELL_EMPTY = 0;
export const CELL_X = 1;
export const CELL_O = 2;

export type TurnStatus = 'in_progress' | 'x_wins' | 'o_wins' | 'draw' | 'unknown';

export interface TurnState {
  isXTurn: boolean;
  moveCount: number;
  status: number;
  statusLabel: TurnStatus;
}

export interface PlayersState {
  playerX?: string;
  playerO?: string;
}

export interface BoardState {
  cells: number[];
  width: number;
  height: number;
}

export function statusToLabel(status: number): TurnStatus {
  switch (status) {
    case STATUS_IN_PROGRESS:
      return 'in_progress';
    case STATUS_X_WINS:
      return 'x_wins';
    case STATUS_O_WINS:
      return 'o_wins';
    case STATUS_DRAW:
      return 'draw';
    default:
      return 'unknown';
  }
}

/**
 * Decode serialized `TurnState` bytes produced by `World::set_typed_observed`.
 *
 * Wire layout from `impl_component!(TurnState, "turnst", Table, { is_x_turn: bool, move_count: u32, status: u32 })`:
 * - byte 0: bool (1 = true, 0 = false)
 * - bytes 1..5: u32 big-endian move_count
 * - bytes 5..9: u32 big-endian status
 */
export function decodeTurnState(data: Uint8Array): TurnState {
  if (data.length < 9) {
    throw new TypeError(`TurnState byte length must be at least 9, got ${data.length}`);
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const isXTurn = data[0] !== 0;
  const moveCount = view.getUint32(1, false);
  const status = view.getUint32(5, false);

  return {
    isXTurn,
    moveCount,
    status,
    statusLabel: statusToLabel(status),
  };
}

/** Format a cell marker to character representation. */
export function formatCell(cell: number): string {
  switch (cell) {
    case CELL_X:
      return 'X';
    case CELL_O:
      return 'O';
    default:
      return '.';
  }
}

/**
 * Render a 2D ASCII board.
 */
export function formatBoardAscii(cells: number[], width = 3, height = 3): string {
  const rows: string[] = [];
  for (let r = 0; r < height; r += 1) {
    const rowCells: string[] = [];
    for (let c = 0; c < width; c += 1) {
      const idx = r * width + c;
      rowCells.push(formatCell(cells[idx] ?? 0));
    }
    rows.push(' ' + rowCells.join(' | ') + ' ');
  }
  const divider = '\n---+---+---\n';
  return rows.join(divider);
}

/**
 * Generate a descriptive action summary comparing previous and current state.
 */
export function describeStateTransition(
  prevTurn?: TurnState,
  currTurn?: TurnState,
  prevBoard?: number[],
  currBoard?: number[],
): string {
  if (!currTurn && !currBoard) return 'State update';

  if (!prevTurn && currTurn?.moveCount === 0) {
    return 'Match initialized: X to move';
  }

  // Find which cell changed
  let cellDesc = '';
  if (prevBoard && currBoard && currBoard.length === prevBoard.length) {
    for (let i = 0; i < currBoard.length; i += 1) {
      if (currBoard[i] !== prevBoard[i]) {
        const marker = formatCell(currBoard[i] ?? 0);
        const row = Math.floor(i / 3);
        const col = i % 3;
        cellDesc = `${marker} placed at cell ${i} (row ${row}, col ${col})`;
        break;
      }
    }
  }

  if (currTurn) {
    if (currTurn.status === STATUS_X_WINS) {
      return cellDesc ? `${cellDesc} - Player X WINS!` : 'Player X WINS!';
    }
    if (currTurn.status === STATUS_O_WINS) {
      return cellDesc ? `${cellDesc} - Player O WINS!` : 'Player O WINS!';
    }
    if (currTurn.status === STATUS_DRAW) {
      return cellDesc ? `${cellDesc} - Game ended in a DRAW` : 'Game ended in a DRAW';
    }
    if (cellDesc) {
      const nextPlayer = currTurn.isXTurn ? 'X' : 'O';
      return `${cellDesc} (Move #${currTurn.moveCount}, next: ${nextPlayer})`;
    }
    const nextPlayer = currTurn.isXTurn ? 'X' : 'O';
    return `Move #${currTurn.moveCount} completed (next: ${nextPlayer})`;
  }

  return cellDesc || 'Board updated';
}
