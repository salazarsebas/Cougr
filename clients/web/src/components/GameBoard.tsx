import type { GameState } from "../game";

export interface GameBoardProps { state: GameState; disabled: boolean; onMove: (position: number) => void; }

export function GameBoard({ state, disabled, onMove }: GameBoardProps) {
  return <div className="board" aria-label="3 by 3 turn-based board">{state.cells.map((value, index) => <button key={index} type="button" disabled={disabled || state.status !== 0 || value !== 0} onClick={() => onMove(index)}>{value === 1 ? "X" : value === 2 ? "O" : ""}</button>)}</div>;
}
