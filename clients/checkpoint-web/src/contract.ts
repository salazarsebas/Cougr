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
import type { ActionResult, MatchState } from "./game";
import { isActionResult, isMatchState } from "./game";

export const TESTNET_RPC_URL = "https://soroban-testnet.stellar.org";
export const TESTNET_PASSPHRASE = Networks.TESTNET;
export const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID?.trim() ?? "";

const server = new Server(TESTNET_RPC_URL);

export class ClientError extends Error {}

export interface Invocation {
  hash: string;
  result: ActionResult;
}

function assertConfigured(): void {
  if (!CONTRACT_ID) throw new ClientError("Set VITE_CONTRACT_ID to the deployed checkpoint contract ID.");
  if (!StrKey.isValidContract(CONTRACT_ID)) throw new ClientError("VITE_CONTRACT_ID is not a valid Soroban contract ID.");
}

async function connectedAddress(): Promise<string> {
  const response = await getAddress();
  if ("error" in response && response.error) throw new ClientError(response.error);
  if (!response.address) throw new ClientError("Freighter did not return a wallet address.");
  return response.address;
}

/**
 * Freighter must be on Stellar Testnet. Mainnet is refused by design: this is
 * a reference client for the checkpoint template, not a production wallet.
 */
export async function requireTestnet(): Promise<string> {
  const network = await getNetwork();
  if (network.networkPassphrase !== TESTNET_PASSPHRASE) {
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

/**
 * Sign with the connected Freighter wallet and submit. A checkpoint action can
 * be rejected by the contract yet still be a successful transaction: the
 * template returns ActionResult { success: false, message } instead of
 * trapping. The simulated return value is carried back so a late dispute
 * surfaces as a rejection, not a crash.
 */
async function signAndSubmit(tx: Transaction, simulation: Api.SimulateTransactionResponse): Promise<Invocation> {
  const rawResult = Api.isSimulationSuccess(simulation) ? simulation.result?.retval : undefined;

  const signed = await signTransaction(tx.toXdr(), { networkPassphrase: TESTNET_PASSPHRASE });
  if ("error" in signed && signed.error) throw new ClientError(signed.error);
  if (!signed.signedTxXdr) throw new ClientError("Freighter did not return a signed transaction.");

  const submitted = await server.sendTransaction(
    TransactionBuilder.fromXdr(signed.signedTxXdr, TESTNET_PASSPHRASE)
  );
  if (submitted.status !== "PENDING") {
    throw new ClientError(`Transaction was not accepted by Soroban RPC: ${submitted.status}.`);
  }

  const finished = await server.pollTransaction(submitted.hash, {
    sleepStrategy: () => 500,
    attempts: 30
  });
  if (finished.status !== "SUCCESS") throw new ClientError(`Transaction failed: ${finished.status}.`);

  const result = rawResult ? scValToNative(rawResult) : null;
  if (!isActionResult(result)) throw new ClientError("The contract returned an unexpected action result.");
  return { hash: submitted.hash, result };
}

async function invoke(address: string, method: string, args: xdr.ScVal[]): Promise<Invocation> {
  const { tx, simulation } = await buildAndSimulate(address, method, args);
  return signAndSubmit(assembleTransaction(tx, simulation).build(), simulation);
}

async function simulateGetter(address: string, method: string): Promise<unknown> {
  const { simulation } = await buildAndSimulate(address, method, []);
  if (!Api.isSimulationSuccess(simulation)) {
    throw new ClientError("Contract getter simulation failed.");
  }
  const result = simulation.result?.retval;
  if (!result) throw new ClientError("Contract getter returned no value.");
  return scValToNative(result);
}

function assertAddressInput(label: string, player: string): xdr.ScVal {
  const trimmed = player.trim();
  if (!StrKey.isValidEd25519PublicKey(trimmed)) {
    throw new ClientError(`${label} must be a valid Stellar account address.`);
  }
  return Address.fromString(trimmed).toScVal();
}

function assertU32(label: string, raw: number): void {
  if (!Number.isSafeInteger(raw) || raw < 0 || raw > 0xffffffff) {
    throw new ClientError(`${label} must be an integer between 0 and 4294967295.`);
  }
}

function assertU64(label: string, raw: string): void {
  if (!/^\d{1,20}$/.test(raw) || BigInt(raw) > 0xffffffffffffffffn) {
    throw new ClientError(`${label} must be a decimal u64 (0 to 18446744073709551615).`);
  }
}

export async function startMatch(player: string): Promise<Invocation> {
  const caller = await requireTestnet();
  assertConfigured();
  return invoke(caller, "start_match", [assertAddressInput("Player", player)]);
}

export async function commitCheckpoint(player: string, tick: string, stateHash: string, score: string): Promise<Invocation> {
  const caller = await requireTestnet();
  assertConfigured();

  const tickValue = Number(tick);
  const scoreValue = Number(score);
  assertAddressInput("Player", player);
  assertU32("Tick", tickValue);
  assertU64("State hash", stateHash);
  assertU32("Score", scoreValue);

  return invoke(caller, "commit_checkpoint", [
    Address.fromString(player.trim()).toScVal(),
    nativeToScVal(tickValue, { type: "u32" }),
    nativeToScVal(BigInt(stateHash), { type: "u64" }),
    nativeToScVal(scoreValue, { type: "u32" })
  ]);
}

export async function disputeCheckpoint(challenger: string, tick: string, claimedHash: string): Promise<Invocation> {
  const caller = await requireTestnet();
  assertConfigured();

  const tickValue = Number(tick);
  assertAddressInput("Challenger", challenger);
  assertU32("Tick", tickValue);
  assertU64("Claimed hash", claimedHash);

  return invoke(caller, "dispute_checkpoint", [
    Address.fromString(challenger.trim()).toScVal(),
    nativeToScVal(tickValue, { type: "u32" }),
    nativeToScVal(BigInt(claimedHash), { type: "u64" })
  ]);
}

export async function finalizeMatch(player: string): Promise<Invocation> {
  const caller = await requireTestnet();
  assertConfigured();
  return invoke(caller, "finalize_match", [assertAddressInput("Player", player)]);
}

export async function getState(): Promise<MatchState> {
  const address = await requireTestnet();
  const raw = await simulateGetter(address, "get_state");
  if (!isMatchState(raw)) throw new ClientError("The contract returned an unexpected match state.");
  return raw;
}
