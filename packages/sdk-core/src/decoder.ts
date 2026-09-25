import { xdr, scValToNative } from '@stellar/stellar-sdk';

export interface TurnBasedState {
  turn: number;
  players: string[];
  active_player: string;
}

/**
 * Fixture decoder for a turn-based game state.
 * Expects a Soroban map ScVal with keys like turn, players, and active_player.
 */
export function decodeTurnBasedState(val: xdr.ScVal): TurnBasedState {
    if (val.switch() !== xdr.ScValType.scvMap()) {
        throw new Error('Expected ScVal to be a map');
    }
    
    const native = scValToNative(val) as Record<string, any>;
    
    return {
        turn: typeof native.turn === 'number' || typeof native.turn === 'bigint' ? Number(native.turn) : 0,
        players: Array.isArray(native.players) ? native.players : [],
        active_player: typeof native.active_player === 'string' ? native.active_player : "",
    };
}
