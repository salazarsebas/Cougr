// ─── Core Domain Types ───────────────────────────────────────────────────────

export type Difficulty = 'Easy' | 'Medium' | 'Expert';

export type ClueType =
  | 'not_same_row'
  | 'not_same_col'
  | 'adjacent'
  | 'not_adjacent'
  | 'same_row'
  | 'same_col';

export interface Suspect {
  id: number;
  name: string;
  /** Hex color used as cell tint and left-border accent */
  color: string;
  /** Short label shown in small UI contexts */
  initials: string;
}

export interface Clue {
  id: string;
  type: ClueType;
  suspectAId: number;
  suspectBId: number;
  description: string;
}

export interface Puzzle {
  id: string;
  title: string;
  description: string;
  gridSize: 4 | 5;
  difficulty: Difficulty;
  suspects: Suspect[];
  clues: Clue[];
  /** Flat row-major solution array (suspect id per cell, 0 = empty) */
  solution: number[];
  creatorAddress: string;
  createdAt: string;
}

export interface PuzzleSummary {
  id: string;
  title: string;
  gridSize: 4 | 5;
  difficulty: Difficulty;
  clueCount: number;
  creatorAddress: string;
}

// ─── Game Session State ───────────────────────────────────────────────────────

export type CellStatus = 'empty' | 'filled' | 'conflict';

export interface Cell {
  suspectId: number | null;
  status: CellStatus;
}

export interface GameState {
  puzzleId: string;
  board: Cell[];
  gridSize: 4 | 5;
  suspects: Suspect[];
  selectedSuspectId: number | null;
  placedSuspectIds: Set<number>;
  isSolved: boolean;
  moveCount: number;
}

// ─── Create Wizard ────────────────────────────────────────────────────────────

export type CreateStep = 'config' | 'solution' | 'clues' | 'review';

export interface CreateWizardState {
  step: CreateStep;
  gridSize: 4 | 5;
  suspects: Suspect[];
  solution: number[];
  clues: Clue[];
}

// ─── Wallet / Auth ────────────────────────────────────────────────────────────

export interface WalletState {
  connected: boolean;
  address: string | null;
}

export type MoveResult =
  | 'Ok'
  | 'RowConflict'
  | 'ColConflict'
  | 'CellOccupied'
  | 'GameAlreadySolved'
  | 'InvalidCoordinates';

export interface ContractCell {
  suspectId: number | null;
}

export interface PlayerState {
  cells: ContractCell[];
  moveCount: number;
  solved: boolean;
}

export interface ContractError {
  message: string;
  code?: number;
}

export interface PuzzleSummaryWithSolvers {
  id: string;
  title: string;
  gridSize: 4 | 5;
  difficulty: Difficulty;
  totalSolvers: number;
  active: boolean;
  creatorAddress: string;
}

export interface PuzzleListResult {
  puzzles: PuzzleSummaryWithSolvers[];
  totalCount: number;
  hasMore: boolean;
}

export interface ContractPuzzleClue {
  id: string;
  type: ClueType;
  suspectAId: number;
  suspectBId: number;
  description: string;
}

export interface ContractPuzzle {
  id: string;
  title: string;
  description: string;
  gridSize: 4 | 5;
  difficulty: Difficulty;
  suspects: Suspect[];
  clues: ContractPuzzleClue[];
  solution: number[];
  creatorAddress: string;
  active: boolean;
}

export interface UseMutationResult<T> {
  mutate: (...args: any[]) => Promise<T>;
  loading: boolean;
  error: string | null;
  data: T | null;
}
