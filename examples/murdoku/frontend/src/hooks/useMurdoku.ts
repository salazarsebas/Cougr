import { usePollar } from './usePollar';
import * as api from '../contract';

export function useMurdoku() {
  const { isAuthenticated, address } = usePollar();
  const wallet = isAuthenticated ? { address } : undefined;

  return {
    submitPuzzle: (payload: any) => api.submitPuzzle(wallet as any, payload),
    getPuzzle: (id: number) => api.getPuzzle(wallet as any, id),
    listPuzzles: (o = 0, l = 20) => api.listPuzzles(wallet as any, o, l),
    startGame: (id: number) => api.startGame(wallet as any, id),
    placeSuspect: (pid: number, sid: number, r: number, c: number) => api.placeSuspect(wallet as any, pid, sid, r, c),
    removeSuspect: (pid: number, r: number, c: number) => api.removeSuspect(wallet as any, pid, r, c),
    getPlayerState: (pid: number) => api.getPlayerState(wallet as any, pid),
    isSolved: (pid: number) => api.isSolved(wallet as any, pid),
  };
}
