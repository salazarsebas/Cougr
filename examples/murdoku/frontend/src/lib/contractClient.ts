import type { CreatorState } from "../types";

export interface SubmitResult {
  puzzleId: string;
}

export async function submitPuzzle(state: CreatorState): Promise<SubmitResult> {
  // TODO: replace with actual Soroban contract invocation
  console.log("Submitting puzzle to chain:", state);
  await new Promise((r) => setTimeout(r, 1500));
  const puzzleId = `PUZZLE-${Math.floor(Math.random() * 100000)}`;
  return { puzzleId };
}
