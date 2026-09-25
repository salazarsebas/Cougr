export type GameStatus = 0 | 1 | 2 | 3;

export interface GameState {
  cells: number[];
  player_x: string;
  player_o: string;
  is_x_turn: boolean;
  move_count: number;
  status: GameStatus;
}

export const STATUS = {
  IN_PROGRESS: 0,
  X_WINS: 1,
  O_WINS: 2,
  DRAW: 3
} as const;

export function moveRejection(message: string): string {
  switch (message) {
    case "notturn": return "It is not your turn.";
    case "occupied": return "That cell is already occupied.";
    case "bounds": return "That move is outside the board.";
    case "notplay": return "The connected wallet is not one of the two players.";
    case "gameover": return "The match is already over.";
    default: return message || "The contract rejected the move.";
  }
}

export function statusLabel(status: GameStatus): string {
  switch (status) {
    case STATUS.X_WINS: return "X wins";
    case STATUS.O_WINS: return "O wins";
    case STATUS.DRAW: return "Draw";
    default: return "In progress";
  }
}

export function cellLabel(value: number): string {
  return value === 1 ? "X" : value === 2 ? "O" : "";
}

export function isGameState(value: unknown): value is GameState {
  if (!value || typeof value !== "object") return false;
  const state = value as Partial<GameState>;
  return (
    Array.isArray(state.cells) &&
    state.cells.length === 9 &&
    state.cells.every((cell) => cell === 0 || cell === 1 || cell === 2) &&
    typeof state.player_x === "string" &&
    typeof state.player_o === "string" &&
    typeof state.is_x_turn === "boolean" &&
    typeof state.move_count === "number" &&
    [0, 1, 2, 3].includes(state.status ?? -1)
  );
}