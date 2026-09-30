// Soroban RPC + Freighter integration for the storageworld template client.
//
// Design mirrors clients/web/src/contract.ts exactly: no custodial keys, no
// packages/sdk-core dependency, testnet-only guard, direct RPC reads.
//
// Contract methods used (storageworld template, working name per issue #330):
//   get_state()                             → MatchState
//   move_entity(player, entity_id, dir)     → MatchState
//
// If #330 lands with different method names, update the string literals in
// simulateGetter() and invoke() below.

import {
  Address,
  Contract,
  Networks,
  Transaction,
  TransactionBuilder,
  nativeToScVal,
  scValToNative,
  StrKey,
  xdr,
} from "@stellar/stellar-sdk";
import { Api, assembleTransaction, Server } from "@stellar/stellar-sdk/rpc";
import { getAddress, getNetwork, signTransaction } from "@stellar/freighter-api";
import type { MatchState, Direction } from "./game";
import { isMatchState } from "./game";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const TESTNET_RPC_URL = "https://soroban-testnet.stellar.org";
export const TESTNET_PASSPHRASE = Networks.TESTNET;

/** Set via .env.local: VITE_CONTRACT_ID=C... */
export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID?.trim() ?? "";

const server = new Server(TESTNET_RPC_URL);

// ---------------------------------------------------------------------------
// Error class
// ---------------------------------------------------------------------------

export class ClientError extends Error {}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function assertConfigured(): void {
  if (!CONTRACT_ID)
    throw new ClientError(
      "Set VITE_CONTRACT_ID to the deployed storageworld contract ID."
    );
  if (!StrKey.isValidContract(CONTRACT_ID))
    throw new ClientError(
      "VITE_CONTRACT_ID is not a valid Soroban contract ID."
    );
}

async function connectedAddress(): Promise<string> {
  const response = await getAddress();
  if ("error" in response && response.error)
    throw new ClientError(response.error);
  if (!response.address)
    throw new ClientError("Freighter did not return a wallet address.");
  return response.address;
}

/** Verify Freighter is on Testnet and return the connected account address. */
export async function requireTestnet(): Promise<string> {
  const network = await getNetwork();
  if (network.networkPassphrase !== TESTNET_PASSPHRASE) {
    throw new ClientError(
      "Freighter must be connected to Stellar Testnet. Mainnet is not supported by this client."
    );
  }
  return connectedAddress();
}

async function buildAndSimulate(
  address: string,
  method: string,
  args: xdr.ScVal[]
) {
  assertConfigured();
  const account = await server.getAccount(address);
  const contract = new Contract(CONTRACT_ID);
  const tx = new TransactionBuilder(account, {
    fee: "100",
    networkPassphrase: TESTNET_PASSPHRASE,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(60)
    .build();

  const simulation = await server.simulateTransaction(tx);
  if ("error" in simulation && simulation.error)
    throw new ClientError(simulation.error);
  return { tx, simulation };
}

async function signAndSubmit(tx: Transaction): Promise<string> {
  const signed = await signTransaction(tx.toXdr(), {
    networkPassphrase: TESTNET_PASSPHRASE,
  });
  if ("error" in signed && signed.error) throw new ClientError(signed.error);
  if (!signed.signedTxXdr)
    throw new ClientError("Freighter did not return a signed transaction.");

  const submitted = await server.sendTransaction(
    TransactionBuilder.fromXdr(signed.signedTxXdr, TESTNET_PASSPHRASE)
  );
  if (submitted.status !== "PENDING") {
    throw new ClientError(
      `Transaction was not accepted by Soroban RPC: ${submitted.status}.`
    );
  }

  const result = await server.pollTransaction(submitted.hash, {
    sleepStrategy: () => 500,
    attempts: 30,
  });
  if (result.status !== "SUCCESS")
    throw new ClientError(`Transaction failed: ${result.status}.`);
  return submitted.hash;
}

async function invoke(
  address: string,
  method: string,
  args: xdr.ScVal[]
): Promise<string> {
  const { tx, simulation } = await buildAndSimulate(address, method, args);
  return signAndSubmit(assembleTransaction(tx, simulation).build());
}

// ---------------------------------------------------------------------------
// Contract reads
// ---------------------------------------------------------------------------

async function simulateGetter(address: string): Promise<unknown> {
  const { simulation } = await buildAndSimulate(address, "get_state", []);
  if (!Api.isSimulationSuccess(simulation)) {
    throw new ClientError("Contract getter simulation failed.");
  }
  const result = simulation.result?.retval;
  if (!result) throw new ClientError("Contract getter returned no value.");
  return scValToNative(result);
}

/** Read the current match state from Soroban RPC. */
export async function getState(): Promise<MatchState> {
  const player = await requireTestnet();
  const raw = await simulateGetter(player);
  if (!isMatchState(raw))
    throw new ClientError(
      "The contract returned an unexpected match state. Check VITE_CONTRACT_ID."
    );
  return raw;
}

// ---------------------------------------------------------------------------
// Contract writes
// ---------------------------------------------------------------------------

/**
 * Submit a move for the given entity.
 *
 * @param entityId  0 or 1 – the entity the caller wants to move.
 * @param direction One of "up" | "down" | "left" | "right".
 * @returns Transaction hash on success.
 */
export async function moveEntity(
  entityId: number,
  direction: Direction
): Promise<string> {
  const player = await requireTestnet();
  assertConfigured();

  if (entityId !== 0 && entityId !== 1) {
    throw new ClientError("Entity id must be 0 or 1.");
  }

  return invoke(player, "move_entity", [
    Address.fromString(player).toScVal(),
    nativeToScVal(entityId, { type: "u32" }),
    nativeToScVal(direction, { type: "symbol" }),
  ]);
}
