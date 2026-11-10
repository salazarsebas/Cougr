export type GridSize = 4 | 5;
export type Difficulty = "Easy" | "Medium" | "Expert";

export const ClueType = {
  SameRow: "SameRow",
  SameColumn: "SameColumn",
  AdjacentRow: "AdjacentRow",
  AdjacentColumn: "AdjacentColumn",
  NotSameRow: "NotSameRow",
  NotSameColumn: "NotSameColumn",
  AbsolutePosition: "AbsolutePosition",
  RelativePosition: "RelativePosition",
} as const;

export type ClueType = (typeof ClueType)[keyof typeof ClueType];

export interface Suspect {
  id: string;
  name: string;
  color: string;
}

export interface Clue {
  id: string;
  type: ClueType;
  primarySuspectId: string;
  secondarySuspectId?: string;
  row?: number;
  col?: number;
}

export interface CreatorState {
  gridSize: GridSize;
  puzzleName: string;
  difficulty: Difficulty;
  suspects: Suspect[];
  solution: string[][];
  clues: Clue[];
}

export const SUSPECT_COLORS = [
  "#e63946","#2a9d8f","#e9c46a","#6a4c93","#f4a261",
  "#457b9d","#8ecae6","#06d6a0","#ef476f","#ffd166",
];

export const CLUE_TYPE_LABELS: Record<ClueType, string> = {
  SameRow: "Same Row",
  SameColumn: "Same Column",
  AdjacentRow: "Adjacent Row",
  AdjacentColumn: "Adjacent Column",
  NotSameRow: "Not Same Row",
  NotSameColumn: "Not Same Column",
  AbsolutePosition: "Absolute Position",
  RelativePosition: "Relative Position",
};

export const CLUE_NEEDS_SECONDARY: ClueType[] = [
  "SameRow","SameColumn","AdjacentRow","AdjacentColumn",
  "NotSameRow","NotSameColumn","RelativePosition",
];

export const CLUE_NEEDS_COORDS: ClueType[] = ["AbsolutePosition"];
