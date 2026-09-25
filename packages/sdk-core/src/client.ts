import { SorobanRpc, TransactionBuilder, Account, Contract, Transaction } from '@stellar/stellar-sdk';
import { ClientConfig, ContractInvocation, Signer, SimulationError } from './types.js';

export class CougrClient {
  private rpc: SorobanRpc.Server;
  private networkPassphrase: string;

  constructor(config: ClientConfig) {
    this.rpc = new SorobanRpc.Server(config.rpcUrl, { allowHttp: true });
    this.networkPassphrase = config.networkPassphrase;
  }

  /**
   * Simulates, assembles, signs, and submits a contract invocation.
   */
  async invoke(
    invocation: ContractInvocation,
    signer: Signer,
    sourceAccountId: string
  ): Promise<SorobanRpc.Api.GetTransactionResponse> {
    const { contractId, method, args } = invocation;

    const sourceAccount = await this.getSourceAccount(sourceAccountId);
    const contract = new Contract(contractId);

    const tx = new TransactionBuilder(sourceAccount, {
      fee: '100', // Base fee, adjusted during assembly
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();

    // 1. Simulate
    const simResult = await this.rpc.simulateTransaction(tx);
    
    if (SorobanRpc.Api.isSimulationError(simResult)) {
        throw new SimulationError(`Simulation failed: ${simResult.error}`, simResult);
    }
    
    // 2. Assemble
    const assembledTx = SorobanRpc.assembleTransaction(tx, this.networkPassphrase, simResult).build();
    
    // 3. Sign using injected signer
    const signedTxXdr = await signer.sign(assembledTx.toXDR());
    
    // 4. Submit
    const transactionToSubmit = TransactionBuilder.fromXDR(signedTxXdr, this.networkPassphrase) as Transaction;
    const submitResult = await this.rpc.sendTransaction(transactionToSubmit);

    if (submitResult.status === 'ERROR') {
       throw new Error(`Submission failed: ${JSON.stringify(submitResult)}`);
    }

    // 5. Wait for the final result
    return await this.waitForTransaction(submitResult.hash);
  }

  private async getSourceAccount(accountId: string): Promise<Account> {
    const accountInfo = await this.rpc.getAccount(accountId);
    return new Account(accountId, accountInfo.sequenceNumber().toString());
  }

  private async waitForTransaction(hash: string): Promise<SorobanRpc.Api.GetTransactionResponse> {
    let result = await this.rpc.getTransaction(hash);
    while (result.status === 'NOT_FOUND') {
      await new Promise(resolve => setTimeout(resolve, 1500));
      result = await this.rpc.getTransaction(hash);
    }
    return result;
  }
}
