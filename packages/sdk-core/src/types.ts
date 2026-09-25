import { xdr } from '@stellar/stellar-sdk';

export interface Signer {
  sign(txXdr: string): Promise<string>;
}

export interface ClientConfig {
  rpcUrl: string;
  networkPassphrase: string;
}

export interface ContractInvocation {
  contractId: string;
  method: string;
  args: xdr.ScVal[];
}

export class SimulationError extends Error {
  constructor(message: string, public result?: any) {
    super(message);
    this.name = 'SimulationError';
  }
}
