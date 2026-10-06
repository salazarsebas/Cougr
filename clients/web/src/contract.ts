import {
  Address,
  Networks,
  StrKey,
  nativeToScVal,
  scValToNative,
} from '@stellar/stellar-sdk';
import { Api } from '@stellar/stellar-sdk/rpc';
import type { xdr } from '@stellar/stellar-sdk';
import {
  ClientError,
  createClientUi,
  type ClientUiConfig,
} from '@cougr/client-ui';
import type { GameState } from './game';
import { isGameState } from './game';

export const TESTNET_RPC_URL = 'https://soroban-testnet.stellar.org';
export const TESTNET_PASSPHRASE = Networks.TESTNET;
export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID?.trim() ?? '';

const config: ClientUiConfig = {
  contractId: CONTRACT_ID,
  rpcUrl: TESTNET_RPC_URL,
  networkPassphrase: TESTNET_PASSPHRASE,
};
const client = createClientUi(config);

export { ClientError };
export const requireTestnet = client.requireTestnet;

function assertContractConfigured(): void {
  if (!CONTRACT_ID.trim()) {
    throw new ClientError('Set VITE_CONTRACT_ID to the deployed turn-based contract ID.');
  }
  if (!StrKey.isValidContract(CONTRACT_ID)) {
    throw new ClientError('VITE_CONTRACT_ID is not a valid Soroban contract ID.');
  }
}

async function simulateGetter(address: string): Promise<unknown> {
  const { simulation } = await client.buildAndSimulate(address, 'get_state', []);
  if (!Api.isSimulationSuccess(simulation)) {
    throw new ClientError('Contract getter simulation failed.');
  }
  const result = simulation.result?.retval;
  if (!result) throw new ClientError('Contract getter returned no value.');
  return scValToNative(result);
}

export async function initGame(playerO: string): Promise<string> {
  const playerX = await requireTestnet();
  assertContractConfigured();
  if (!StrKey.isValidEd25519PublicKey(playerO.trim())) {
    throw new ClientError('Player O must be a valid Stellar account address.');
  }

  const playerOAddress = Address.fromString(playerO.trim());
  if (playerO.trim() === playerX) {
    throw new ClientError('Player O must be different from the connected wallet.');
  }

  const args: xdr.ScVal[] = [
    Address.fromString(playerX).toScVal(),
    playerOAddress.toScVal(),
  ];
  return client.invoke(playerX, 'init_game', args);
}

export async function makeMove(position: number): Promise<string> {
  const player = await requireTestnet();
  assertContractConfigured();
  if (!Number.isInteger(position) || position < 0 || position > 8) {
    throw new ClientError('Move position must be between 0 and 8.');
  }

  return client.invoke(player, 'make_move', [
    Address.fromString(player).toScVal(),
    nativeToScVal(position, { type: 'u32' }),
  ]);
}

export async function getState(): Promise<GameState> {
  const player = await requireTestnet();
  const raw = await simulateGetter(player);
  if (!isGameState(raw)) throw new ClientError('The contract returned an unexpected game state.');
  return raw;
}
