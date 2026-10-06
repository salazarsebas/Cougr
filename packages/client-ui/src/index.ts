import {
  Address,
  Contract,
  Networks,
  Transaction,
  TransactionBuilder,
  nativeToScVal,
  type xdr,
} from '@stellar/stellar-sdk';
import { Api, assembleTransaction, Server } from '@stellar/stellar-sdk/rpc';
import { getAddress, getNetwork, signTransaction } from '@stellar/freighter-api';

export class ClientError extends Error {}

export interface ClientUiConfig {
  contractId: string;
  rpcUrl: string;
  networkPassphrase?: string;
}

export interface MessageBannerOptions {
  error?: string;
  notice?: string;
}

export interface ConnectPanelOptions {
  address: string;
  busy: boolean;
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (character) => {
    const entities: Record<string, string> = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#39;',
    };
    return entities[character];
  });
}

/** Framework-independent connect panel markup. Use a delegated click handler for its button. */
export function renderConnectPanel({ address, busy }: ConnectPanelOptions): string {
  const connected = Boolean(address);
  const wallet = connected
    ? `${escapeHtml(address.slice(0, 8))}…${escapeHtml(address.slice(-6))}`
    : 'Not connected';

  return `<div class="row"><div><span class="label">Wallet</span><code>${wallet}</code></div><button type="button" data-client-ui-connect${busy ? ' disabled' : ''}>${connected ? 'Reconnect' : 'Connect Freighter'}</button></div><p class="network">Network: Stellar Testnet only</p>`;
}

/** Framework-independent accessible error or notice banner markup. */
export function renderMessageBanner({ error, notice }: MessageBannerOptions): string {
  if (error) return `<div class="error" role="alert">${escapeHtml(error)}</div>`;
  if (notice) return `<div class="empty" role="status">${escapeHtml(notice)}</div>`;
  return '';
}

function passphraseFor(config: ClientUiConfig): string {
  return config.networkPassphrase ?? Networks.TESTNET;
}

function assertConfigured(config: ClientUiConfig): void {
  if (!config.contractId.trim() || !config.rpcUrl.startsWith('http')) {
    throw new ClientError('contractId and rpcUrl are required.');
  }
}

/** Read the connected wallet address from Freighter. */
export async function connectedAddress(): Promise<string> {
  const response = await getAddress();
  if ('error' in response && response.error) throw new ClientError(response.error);
  if (!response.address) throw new ClientError('Freighter did not return a wallet address.');
  return response.address;
}

/** Require Freighter to be connected to Testnet, then return its address. */
export async function requireTestnet(
  networkPassphrase: string = Networks.TESTNET
): Promise<string> {
  const network = await getNetwork();
  if (network.networkPassphrase !== networkPassphrase) {
    throw new ClientError('Freighter must be connected to Stellar Testnet. Mainnet is not supported by this client.');
  }
  return connectedAddress();
}

export async function buildAndSimulate(
  config: ClientUiConfig,
  address: string,
  method: string,
  args: xdr.ScVal[]
) {
  assertConfigured(config);
  const passphrase = passphraseFor(config);
  const server = new Server(config.rpcUrl);
  const account = await server.getAccount(address);
  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: passphrase,
  })
    .addOperation(new Contract(config.contractId).call(method, ...args))
    .setTimeout(60)
    .build();

  const simulation = await server.simulateTransaction(tx);
  if ('error' in simulation && simulation.error) throw new ClientError(simulation.error);
  return { tx, simulation, server };
}

export async function signAndSubmit(
  config: ClientUiConfig,
  tx: Transaction,
  server: Server = new Server(config.rpcUrl)
): Promise<string> {
  const signed = await signTransaction(tx.toXdr(), {
    networkPassphrase: passphraseFor(config),
  });
  if ('error' in signed && signed.error) throw new ClientError(signed.error);
  if (!signed.signedTxXdr) throw new ClientError('Freighter did not return a signed transaction.');

  const submitted = await server.sendTransaction(
    TransactionBuilder.fromXdr(signed.signedTxXdr, passphraseFor(config))
  );
  if (submitted.status !== 'PENDING') {
    throw new ClientError(`Transaction was not accepted by Soroban RPC: ${submitted.status}.`);
  }

  const result = await server.pollTransaction(submitted.hash, {
    sleepStrategy: () => 500,
    attempts: 30,
  });
  if (result.status !== 'SUCCESS') throw new ClientError(`Transaction failed: ${result.status}.`);
  return submitted.hash;
}

export async function invoke(
  config: ClientUiConfig,
  address: string,
  method: string,
  args: xdr.ScVal[]
): Promise<string> {
  const { tx, simulation, server } = await buildAndSimulate(config, address, method, args);
  return signAndSubmit(config, assembleTransaction(tx, simulation).build(), server);
}

export function createClientUi(config: ClientUiConfig) {
  return {
    connectedAddress,
    requireTestnet: () => requireTestnet(passphraseFor(config)),
    buildAndSimulate: (address: string, method: string, args: xdr.ScVal[]) =>
      buildAndSimulate(config, address, method, args),
    signAndSubmit: (tx: Transaction, server?: Server) => signAndSubmit(config, tx, server),
    invoke: (address: string, method: string, args: xdr.ScVal[]) =>
      invoke(config, address, method, args),
  };
}

export { Address, nativeToScVal };
export type { xdr };
