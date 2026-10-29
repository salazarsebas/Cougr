// Minimal contract client stubs for Murdoku.
// These wrappers expect a connected wallet from Pollar and will throw
// NotAuthenticatedError when no wallet is present.

export class NotAuthenticatedError extends Error {}

interface WalletLike {
  address?: string | null;
  signTransaction?: (...args: any[]) => Promise<any>;
}

function requireAuth(wallet?: WalletLike) {
  if (!wallet || !wallet.address) throw new NotAuthenticatedError('Not authenticated');
}

export async function submitPuzzle(wallet: WalletLike | undefined, payload: any): Promise<number> {
  requireAuth(wallet);
  // TODO: implement actual Soroban transaction via Pollar wallet
  return Promise.resolve(1);
}

export async function getPuzzle(wallet: WalletLike | undefined, puzzleId: number): Promise<any> {
  // Public read; may not require signing
  return Promise.resolve(null);
}

export async function listPuzzles(wallet: WalletLike | undefined, offset = 0, limit = 20): Promise<any[]> {
  return Promise.resolve([]);
}

export async function startGame(wallet: WalletLike | undefined, puzzleId: number): Promise<void> {
  requireAuth(wallet);
  return Promise.resolve();
}

export async function placeSuspect(wallet: WalletLike | undefined, puzzleId: number, suspectId: number, row: number, col: number): Promise<void> {
  requireAuth(wallet);
  return Promise.resolve();
}

export async function removeSuspect(wallet: WalletLike | undefined, puzzleId: number, row: number, col: number): Promise<void> {
  requireAuth(wallet);
  return Promise.resolve();
}

export async function getPlayerState(wallet: WalletLike | undefined, puzzleId: number): Promise<any> {
  requireAuth(wallet);
  return Promise.resolve(null);
}

export async function isSolved(wallet: WalletLike | undefined, puzzleId: number): Promise<boolean> {
  return Promise.resolve(false);
}
