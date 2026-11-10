import type { GridSize } from "../types";

export function isConflict(
  solution: string[][],
  row: number,
  col: number,
  suspectId: string,
  gridSize: GridSize
): boolean {
  for (let c = 0; c < gridSize; c++) {
    if (c !== col && solution[row][c] === suspectId) return true;
  }
  for (let r = 0; r < gridSize; r++) {
    if (r !== row && solution[r][col] === suspectId) return true;
  }
  return false;
}

export function isCompleteSolution(solution: string[][], gridSize: GridSize): boolean {
  for (let r = 0; r < gridSize; r++)
    for (let c = 0; c < gridSize; c++)
      if (!solution[r][c]) return false;
  return true;
}

export function emptyGrid(size: GridSize): string[][] {
  return Array.from({ length: size }, () => Array(size).fill(""));
}
