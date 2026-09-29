import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { WalletService } from '../src/services/wallet-service';
import { configurePlayLoop } from '../src/studio/play-loop';
import { TurnBasedConfig } from '../src/types/config';
import { GameState } from '../src/types/game-state';
import { setupMSWServer } from './msw-server';
import { waitForFinality } from '../src/utils/finality';

const config: TurnBasedConfig = {
  players: ['alice', 'bob'],
  boardSize: 3,
};

describe('Play Loop (Real Services)', () => {
  let walletService: WalletService;
  let playLoop: ReturnType<typeof configurePlayLoop>;

  beforeAll(async () => {
    if (!process.env.USE_REAL_SERVICES) {
      throw new Error('USE_REAL_SERVICES must be true for real-service tests');
    }

    walletService = new WalletService({
      baseUrl: 'http://localhost:8000',
      rpcUrl: process.env.RPC_URL,
      friendbotUrl: process.env.FRIENDBOT_URL,
    });

    playLoop = configurePlayLoop(walletService);
  });

  it('should complete a full game with real services', async () => {
    const gameId = await playLoop.startGame(config);
    expect(gameId).toBeDefined();

    // Wait for initial state
    const initialState = await playLoop.getGameState(gameId);
    expect(initialState).toMatchObject({
      players: config.players,
      board: expect.any(Array),
    });

    // Simulate moves (with retries for transient errors)
    for (let i = 0; i < 5; i++) {
      const move = { row: i % 3, col: Math.floor(i / 3) };
      await playLoop.makeMove(gameId, config.players[i % 2], move);

      // Wait for finality
      await waitForFinality(walletService, gameId, i + 1);
    }

    // Verify final state
    const finalState = await playLoop.getGameState(gameId);
    expect(finalState).toMatchObject({
      status: 'completed' as const,
      winner: expect.any(String),
    });
  }, 30000); // 30s timeout for real-service tests

  afterAll(async () => {
    await playLoop?.cleanup();
  });
});

// Helper: Retry with exponential backoff for transient RPC errors
async function waitForFinality(walletService: WalletService, gameId: string, turn: number) {
  let retries = 0;
  const maxRetries = 5;

  while (retries < maxRetries) {
    try {
      const state = await walletService.getGameState(gameId);
      if (state.turn === turn) return;
    } catch (err) {
      if (retries === maxRetries - 1) throw err;
    }

    await new Promise(resolve => setTimeout(resolve, 1000 * (retries + 1)));
    retries++;
  }
}