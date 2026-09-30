import { test, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';

// ============================================================================
// Types
// ============================================================================
export interface TurnBasedConfig {
  board_width: number; // 3..=8
  board_height: number; // 3..=8
  win_length: number; // 3..=min(board_width, board_height)
  first_player: "x" | "o";
  triggerRpcError?: boolean; // For testing RPC error
}

export interface GameState {
  cells: (string | null)[];
  player_x: string | null;
  player_o: string | null;
  whose_turn: "x" | "o";
  move_count: number;
  status: number; // 0 in progress, 1 X wins, 2 O wins, 3 draw
}

// ============================================================================
// Fake HTTP Shapes Documentation (Next to the service issue's contract)
// ============================================================================
// 1. GET /friendbot?addr=<address>
//    - Real behavior: Funds an account.
//    - Fake shape: returns 200 OK or 400 Bad Request on failure.
//
// 2. POST /reconfigure
//    - Real behavior: Submits a transaction to reconfigure the game contract.
//    - Fake shape: Expects body `TurnBasedConfig`. Returns 200 OK or 400 Bad Request for typed errors (illegal config), or 500 for RPC error.
//
// 3. GET /state
//    - Real behavior: Fetches the current state of the game from the contract.
//    - Fake shape: Returns 200 OK with `GameState`.
//
// 4. POST /rpc/move
//    - Real behavior: Submits a transaction to make a move.
//    - Fake shape: Expects body { index, forceStatus, triggerRpcError }. Returns 200 OK or 500 for RPC error.
// ============================================================================

const baseUrl = process.env.USE_REAL_SERVICES ? 'http://localhost:8000' : 'http://fake-service.local';

let mockState: GameState = {
  cells: Array(9).fill(null),
  player_x: null,
  player_o: null,
  whose_turn: "x",
  move_count: 0,
  status: 0
};

export const handlers = [
  http.get(`${baseUrl}/friendbot`, ({ request }) => {
    const url = new URL(request.url);
    if (url.searchParams.get('addr') === 'FAIL') {
      return HttpResponse.json({ error: 'friendbot failed' }, { status: 400 });
    }
    return HttpResponse.json({ ok: true });
  }),
  
  http.post(`${baseUrl}/reconfigure`, async ({ request }) => {
    const config = await request.json() as TurnBasedConfig;
    if (config.board_width < 3 || config.board_width > 8 || 
        config.board_height < 3 || config.board_height > 8 ||
        config.win_length < 3 || config.win_length > Math.min(config.board_width, config.board_height)) {
       return HttpResponse.json({ error: 'Illegal config' }, { status: 400 });
    }
    if (config.triggerRpcError) {
       return HttpResponse.json({ error: 'RPC error' }, { status: 500 });
    }
    
    mockState = {
      cells: Array(config.board_width * config.board_height).fill(null),
      player_x: 'PlayerX',
      player_o: 'PlayerO',
      whose_turn: config.first_player,
      move_count: 0,
      status: 0
    };
    return HttpResponse.json({ ok: true });
  }),

  http.get(`${baseUrl}/state`, () => {
    return HttpResponse.json(mockState);
  }),
  
  http.post(`${baseUrl}/rpc/move`, async ({ request }) => {
    const move = await request.json() as any;
    if (move.triggerRpcError) {
      return HttpResponse.json({ error: 'RPC error' }, { status: 500 });
    }
    
    mockState.cells[move.index] = mockState.whose_turn;
    mockState.move_count++;
    mockState.whose_turn = mockState.whose_turn === 'x' ? 'o' : 'x';
    
    if (move.forceStatus !== undefined) {
       mockState.status = move.forceStatus;
    }
    
    return HttpResponse.json({ ok: true });
  })
];

const server = setupServer(...handlers);

// ============================================================================
// Service Abstractions
// ============================================================================
class Editor {
   static emitConfig(config: TurnBasedConfig) {
      return config;
   }
}

class WalletService {
   static async fundAccount(address: string) {
      const res = await fetch(`${baseUrl}/friendbot?addr=${address}`);
      if (!res.ok) throw new Error("Friendbot failure");
   }
   static async reconfigure(config: TurnBasedConfig) {
      const res = await fetch(`${baseUrl}/reconfigure`, {
         method: 'POST', body: JSON.stringify(config)
      });
      if (!res.ok) {
          const err = await res.json();
          throw new Error(err.error || "RPC failure");
      }
   }
   static async makeMove(index: number, opts: any = {}) {
      const res = await fetch(`${baseUrl}/rpc/move`, {
         method: 'POST', body: JSON.stringify({ index, ...opts })
      });
      if (!res.ok) {
          const err = await res.json();
          throw new Error(err.error || "RPC failure");
      }
   }
   static async getState(): Promise<GameState> {
      const res = await fetch(`${baseUrl}/state`);
      if (!res.ok) throw new Error("RPC failure");
      return await res.json();
   }
}

class Renderer {
   static render(state: GameState) {
      if (state.status === 1) return "X wins";
      if (state.status === 2) return "O wins";
      if (state.status === 3) return "Draw";
      return "In progress";
   }
}

// ============================================================================
// Tests
// ============================================================================
beforeAll(() => {
  if (!process.env.USE_REAL_SERVICES) {
    server.listen({ onUnhandledRequest: 'error' });
  }
});
afterEach(() => {
  if (!process.env.USE_REAL_SERVICES) {
    server.resetHandlers();
  }
});
afterAll(() => {
  if (!process.env.USE_REAL_SERVICES) {
    server.close();
  }
});

test('happy path', async () => {
  const config = Editor.emitConfig({
    board_width: 3, board_height: 3, win_length: 3, first_player: "x"
  });
  await WalletService.fundAccount("SUCCESS");
  await WalletService.reconfigure(config);
  
  const state = await WalletService.getState();
  expect(state.status).toBe(0); // In progress
  expect(Renderer.render(state)).toBe("In progress");
});

test('illegal config', async () => {
  const config = Editor.emitConfig({
    board_width: 2, board_height: 3, win_length: 3, first_player: "x" // width < 3
  });
  await expect(WalletService.reconfigure(config)).rejects.toThrow("Illegal config");
});

test('friendbot failure', async () => {
  await expect(WalletService.fundAccount("FAIL")).rejects.toThrow("Friendbot failure");
});

test('RPC failure', async () => {
  const config = Editor.emitConfig({
    board_width: 3, board_height: 3, win_length: 3, first_player: "x", triggerRpcError: true
  });
  await expect(WalletService.reconfigure(config)).rejects.toThrow("RPC error");
});

test('win', async () => {
  const config = Editor.emitConfig({
    board_width: 3, board_height: 3, win_length: 3, first_player: "x"
  });
  await WalletService.reconfigure(config);
  
  // Simulate moves leading to a win
  await WalletService.makeMove(0, { forceStatus: 1 }); // X wins
  
  const state = await WalletService.getState();
  expect(state.status).toBe(1);
  expect(Renderer.render(state)).toBe("X wins");
});

test('draw', async () => {
  const config = Editor.emitConfig({
    board_width: 3, board_height: 3, win_length: 3, first_player: "x"
  });
  await WalletService.reconfigure(config);
  
  // Simulate a draw
  await WalletService.makeMove(0, { forceStatus: 3 }); // Draw
  
  const state = await WalletService.getState();
  expect(state.status).toBe(3);
  expect(Renderer.render(state)).toBe("Draw");
});

test('non-3x3 board', async () => {
  const config = Editor.emitConfig({
    board_width: 4, board_height: 5, win_length: 4, first_player: "o"
  });
  await WalletService.reconfigure(config);
  
  const state = await WalletService.getState();
  expect(state.cells.length).toBe(20);
  expect(state.whose_turn).toBe("o");
  expect(Renderer.render(state)).toBe("In progress");
});
