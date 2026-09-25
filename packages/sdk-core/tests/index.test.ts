import { describe, it, expect, vi } from 'vitest';
import { nativeToScVal, xdr, SorobanRpc } from '@stellar/stellar-sdk';
import { decodeTurnBasedState } from '../src/decoder.js';
import { CougrClient } from '../src/client.js';
import { SimulationError } from '../src/types.js';

describe('Decoder', () => {
    it('successfully decodes a turn-based state', () => {
        const mockState = {
            turn: 42,
            players: ['player1', 'player2'],
            active_player: 'player1'
        };
        const scVal = nativeToScVal(mockState);
        const decoded = decodeTurnBasedState(scVal);
        
        expect(decoded.turn).toBe(42);
        expect(decoded.players).toEqual(['player1', 'player2']);
        expect(decoded.active_player).toBe('player1');
    });

    it('throws when not a map', () => {
        const scVal = nativeToScVal(42);
        expect(() => decodeTurnBasedState(scVal)).toThrow('Expected ScVal to be a map');
    });
});

describe('Client Simulation', () => {
    it('surfaces a structured error when simulation fails', async () => {
        const client = new CougrClient({
            rpcUrl: 'https://soroban-testnet.stellar.org',
            networkPassphrase: 'Test SDF Network ; September 2015'
        });

        // Mock rpc simulateTransaction
        vi.spyOn(client['rpc'], 'simulateTransaction').mockResolvedValue({
            error: 'Contract not found',
            events: [],
            latestLedger: 100
        } as unknown as SorobanRpc.Api.SimulateTransactionErrorResponse);
        
        // Mock getAccount to prevent network call
        vi.spyOn(client['rpc'], 'getAccount').mockResolvedValue({
            sequenceNumber: () => '1'
        } as any);

        const signer = { sign: vi.fn() };
        
        await expect(client.invoke({
            contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM',
            method: 'some_method',
            args: []
        }, signer, 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF'))
        .rejects
        .toThrow(SimulationError);
    });
});
