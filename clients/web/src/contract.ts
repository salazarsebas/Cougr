import {
  Address,
  Contract,
  Networks,
  Transaction,
  TransactionBuilder,
  nativeToScVal,
  scValToNative,
  StrKey,
  xdr
} from "@stellar/stellar-sdk";
import { assembleTransaction, Server } from "@stellar/stellar-sdk/rpc";
import { getAddress, getNetwork, signTransaction } from "@stellar/freighter-api";
import type { GameState } from "./game";
import { isGameState } from "./game";

export const TESTNET_RPC_URL = "https://soroban-testnet.stellar.org";
export const TESTNET_PASSPHRASE = Networks.TESTNET;
export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID?.trim() ?? "";

const server = new Server(TESTNET_RPC_URL);

export class ClientError extends Error {}

function assertConfigured(): void {
  if (!CONTRACT_ID) throw new ClientError("Set VITE_CONTRACT_ID to the deployed turn-based contract ID.");
  if (!StrKey.isValidContract(CONTRACT_ID)) throw new ClientError("VITE_CONTRACT_ID is not a valid Soroban contract ID.");
}

async function connectedAddress(): Promise<string> {
  const response = await getAddress();
  if ("error" in response && response.error) throw new ClientError(response.error);
  if (!response.address) throw new ClientError("Freighter did not return a wallet address.");
  return response.address;
}

export async function requireTestnet(): Promise<string> {
  const network = await getNetwork();
  if (network.passphrase !== TESTNET_PASSPHRASE) {
    throw new ClientError("Freighter must be connected to Stellar Testnet. Mainnet is not supported by this client.");
  }
  return connectedAddress();
}

async function buildAndSimulate(address: string, method: string, args: xdr.ScVal[]) {
  assertConfigured();
  const account = await server.getAccount(address);
  const contract = new Contract(CONTRACT_ID);
  const tx = new TransactionBuilder(account, {
    fee: "100",
    networkPassphrase: TESTNET_PASSPHRASE
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(60)
    .build();

  const simulation = await server.simulateTransaction(tx);
  if ("error" in simulation && simulation.error) throw new ClientError(simulation.error);
  return { tx, simulation };
}

async function signAndSubmit(tx: Transaction): Promise<string> {
  const signed = await signTransaction(tx.toXdr(), { networkPassphrase: TESTNET_PASSPHRASE });
  if ("error" in signed && signed.error) throw new ClientError(signed.error);
  if (!signed.signedTxXdr) throw new ClientError("Freighter did not return a signed transaction.");

  const submitted = await server.sendTransaction(
    TransactionBuilder.fromXdr(signed.signedTxXdr, TESTNET_PASSPHRASE)
  );
  if (submitted.status !== "PENDING") {
    throw new ClientError(`Transaction was not accepted by Soroban RPC: ${submitted.status}.`);
  }

  const result = await server.pollTransaction(submitted.hash, {
    sleepStrategy: () => 500,
    attempts: 30
  });
  if (result.status !== "SUCCESS") throw new ClientError(`Transaction failed: ${result.status}.`);
  return submitted.hash;
}

async function invoke(address: string, method: string, args: xdr.ScVal[]): Promise<string> {
  const { tx, simulation } = await buildAndSimulate(address, method, args);
  return signAndSubmit(assembleTransaction(tx, simulation).build());
}

async function simulateGetter(address: string): Promise<unknown> {
  const { simulation } = await buildAndSimulate(address, "get_state", []);
  const result = simulation.result?.[0]?.xdr;
  if (!result) throw new ClientError("Contract getter returned no value.");
  return scValToNative(xdr.ScVal.fromXdr(result, "base64"));
}

export async function initGame(playerO: string): Promise<string> {
  const playerX = await requireTestnet();
  assertConfigured();

  if (!StrKey.isValidEd25519PublicKey(playerO.trim())) {
    throw new ClientError("Player O must be a valid Stellar account address.");
  }

  const playerOAddress = Address.fromString(playerO.trim());
  if (playerO.trim() === playerX) {
    throw new ClientError("Player O must be different from the connected wallet.");
  }

  return invoke(playerX, "init_game", [
    Address.fromString(playerX).toScVal(),
    playerOAddress.toScVal()
  ]);
}

export async function makeMove(position: number): Promise<string> {
  const player = await requireTestnet();
  assertConfigured();

  if (!Number.isInteger(position) || position < 0 || position > 8) {
    throw new ClientError("Move position must be between 0 and 8.");
  }

  return invoke(player, "make_move", [
    Address.fromString(player).toScVal(),
    nativeToScVal(position, { type: "u32" })
  ]);
}

export async function getState(): Promise<GameState> {
  const player = await requireTestnet();
  const raw = await simulateGetter(player);
  if (!isGameState(raw)) throw new ClientError("The contract returned an unexpected game state.");
  return raw;
}