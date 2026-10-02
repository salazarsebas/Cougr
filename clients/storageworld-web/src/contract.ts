// Freighter integration and Soroban RPC calls for the StorageWorld client.
//
// Mirror of clients/web/src/contract.ts adapted for the storageworld template.
// No custodial keys. No packages/sdk-core dependency. Testnet only.
//
// Method names (init_match, make_move, get_state) are written against the
// interface documented in issue #330 / PR #358. Confirm against the final
// template README when that PR merges and update CONTRACT_METHOD_* constants
// below if the names differ.

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
import { Api, assembleTransaction, Server } from "@stellar/stellar-sdk/rpc";
import { getAddress, getNetwork, signTransaction } from "@stellar/freighter-api";
import type { MatchState } from "./game";
import { isMatchState } from "./game";

// ── Config ────────────────────────────────────────────────────────────────────

export const TESTNET_RPC_URL = "https://soroban-testnet.stellar.org";
export const TESTNET_PASSPHRASE = Networks.TESTNET;
export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID?.trim() ?? "";

// Working method names. Update these if the storageworld template lands with
// different names (see issue #330 / PR #358).
const CONTRACT_METHOD_INIT = "init_match";
const CONTRACT_METHOD_MOVE = "make_move";
const CONTRACT_METHOD_GET_STATE = "get_state";

const server = new Server(TESTNET_RPC_URL);

// ── Error type ────────────────────────────────────────────────────────────────

export class ClientError extends Error {}

// ── Guards ────────────────────────────────────────────────────────────────────

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
  if ("error" in response && response.error) throw new ClientError(response.error);
  if (!response.address)
    throw new ClientError("Freighter did not return a wallet address.");
  return response.address;
}

/** Rejects non-Testnet networks. Returns the connected wallet address. */
export async function requireTestnet(): Promise<string> {
  const network = await getNetwork();
  if (network.networkPassphrase !== TESTNET_PASSPHRASE) {
    throw new ClientError(
      "Freighter must be connected to Stellar Testnet. Mainnet is not supported by this client."
    );
  }
  return connectedAddress();
}

// ── Low-level RPC helpers ────────────────────────────────────────────────────

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
    networkPassphrase: TESTNET_PASSPHRASE
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
    networkPassphrase: TESTNET_PASSPHRASE
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
    attempts: 30
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

async function simulateGetter(address: string): Promise<unknown> {
  const { simulation } = await buildAndSimulate(
    address,
    CONTRACT_METHOD_GET_STATE,
    []
  );
  if (!Api.isSimulationSuccess(simulation)) {
    throw new ClientError("Contract getter simulation failed.");
  }
  const result = simulation.result?.retval;
  if (!result) throw new ClientError("Contract getter returned no value.");
  return scValToNative(result);
}

// ── Public API ────────────────────────────────────────────────────────────────

/**
 * Initialise a new match. The connected wallet becomes the first entity owner;
 * `secondPlayer` becomes the second. The template requires at least two
 * entities so a partial update (one entity's field changes, the other does not)
 * is visible in the UI.
 */
export async function initMatch(secondPlayer: string): Promise<string> {
  const firstPlayer = await requireTestnet();
  assertConfigured();

  if (!StrKey.isValidEd25519PublicKey(secondPlayer.trim())) {
    throw new ClientError(
      "Second player must be a valid Stellar account address."
    );
  }
  if (secondPlayer.trim() === firstPlayer) {
    throw new ClientError(
      "The second player must be a different address from the connected wallet."
    );
  }

  return invoke(firstPlayer, CONTRACT_METHOD_INIT, [
    Address.fromString(firstPlayer).toScVal(),
    Address.fromString(secondPlayer.trim()).toScVal()
  ]);
}

/**
 * Submit a move for the connected wallet's entity. `entityId` is the u32
 * contract id for the entity that the caller owns.
 *
 * A StorageWorld move only rewrites the components that actually changed, so
 * the other entity's ledger entry is left untouched – that is the partial
 * update this client exists to demonstrate.
 */
export async function makeMove(entityId: number): Promise<string> {
  const player = await requireTestnet();
  assertConfigured();

  if (!Number.isInteger(entityId) || entityId < 0) {
    throw new ClientError("Entity id must be a non-negative integer.");
  }

  return invoke(player, CONTRACT_METHOD_MOVE, [
    Address.fromString(player).toScVal(),
    nativeToScVal(entityId, { type: "u32" })
  ]);
}

/**
 * Read the full match state directly from Soroban RPC (simulated read – no
 * transaction fee, no wallet signature required beyond identifying the
 * account used to build the simulation envelope).
 */
export async function getState(): Promise<MatchState> {
  const player = await requireTestnet();
  const raw = await simulateGetter(player);
  if (!isMatchState(raw))
    throw new ClientError(
      "The contract returned an unexpected match state shape."
    );
  return raw;
}
