import express from 'express';
import cors from 'cors';
import crypto from 'crypto';
import { Keypair } from '@stellar/stellar-sdk';
import { rpc, TransactionBuilder, Networks, TimeoutInfinite, xdr, Contract, nativeToScVal } from '@stellar/stellar-sdk';

// This service is testnet-only.
export function createServer(options = {}) {
  const app = express();
  app.use(cors());
  app.use(express.json());

  // Mocks can be passed for testing
  const serverRpc = options.rpc || new rpc.Server("https://soroban-testnet.stellar.org");
  const networkPassphrase = options.networkPassphrase || Networks.TESTNET;
  const contractId = options.contractId || 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM'; // mock fixture contract

  // In-memory store for session keys (expiry logic could be added)
  const sessions = new Map();

  // Periodic sweep to enforce session expiry
  const sweepInterval = setInterval(() => {
    const now = Date.now();
    for (const [id, session] of sessions.entries()) {
      if (now > session.expiresAt) {
        sessions.delete(id);
      }
    }
  }, 1000 * 60 * 5); // sweep every 5 minutes
  
  if (sweepInterval.unref) {
    sweepInterval.unref();
  }

  app.post('/api/session', async (req, res) => {
    try {
      const config = req.body;
      
      // Validate config
      const bw = config.board_width ?? 3;
      const bh = config.board_height ?? 3;
      const wl = config.win_length ?? 3;
      const fp = config.first_player ?? 'x';

      if (!Number.isInteger(bw) || bw < 3 || bw > 8) {
        return res.status(400).json({ error: 'board_width out of bounds' });
      }
      if (!Number.isInteger(bh) || bh < 3 || bh > 8) {
        return res.status(400).json({ error: 'board_height out of bounds' });
      }
      const minDim = Math.min(bw, bh);
      if (!Number.isInteger(wl) || wl < 3 || wl > minDim) {
        return res.status(400).json({ error: 'win_length out of bounds' });
      }
      if (fp !== 'x' && fp !== 'o') {
        return res.status(400).json({ error: 'first_player must be x or o' });
      }

      // Generate session keypair
      const keypair = Keypair.random();
      const publicKey = keypair.publicKey();

      // Fund with friendbot
      const friendbotUrl = options.friendbotUrl || `https://friendbot.stellar.org/?addr=${encodeURIComponent(publicKey)}`;
      const fundRes = await fetch(friendbotUrl);
      if (!fundRes.ok) {
        return res.status(500).json({ error: 'friendbot limit or failure' });
      }

      // Store session securely with an opaque token
      const sessionId = crypto.randomBytes(32).toString('hex');
      sessions.set(sessionId, {
        keypair,
        expiresAt: Date.now() + 1000 * 60 * 60, // 1 hour
      });

      // Submit reconfigure to deployed match
      try {
        const sourceAccount = await serverRpc.getAccount(publicKey);
        const contract = new Contract(contractId);
        
        // TurnBasedConfig is a struct. We encode it as an ScMap.
        // Map keys must be sorted alphabetically
        const turnBasedConfigVal = xdr.ScVal.scvMap([
          new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol('board_height'), val: nativeToScVal(bh, { type: 'u32' }) }),
          new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol('board_width'), val: nativeToScVal(bw, { type: 'u32' }) }),
          new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol('first_player'), val: xdr.ScVal.scvVec([xdr.ScVal.scvSymbol(fp === 'x' ? 'X' : 'O')]) }),
          new xdr.ScMapEntry({ key: xdr.ScVal.scvSymbol('win_length'), val: nativeToScVal(wl, { type: 'u32' }) })
        ]);
        
        const tx = new TransactionBuilder(sourceAccount, {
          fee: '10000',
          networkPassphrase,
        })
          .addOperation(contract.call("init", turnBasedConfigVal))
          .setTimeout(TimeoutInfinite)
          .build();

        const preparedTx = await serverRpc.prepareTransaction(tx);
        preparedTx.sign(keypair);

        const txResp = await serverRpc.sendTransaction(preparedTx);
        if (txResp.status === "ERROR") {
          throw new Error(`Tx submitted with error: ${txResp.errorResultXdr}`);
        }
      } catch (err) {
        return res.status(502).json({ error: 'rpc failure', detail: err.message });
      }

      // Do NOT send the keypair.secret() to the client!
      res.json({ sessionId, contractId });
    } catch (err) {
      res.status(500).json({ error: 'internal server error' });
    }
  });

  return app;
}
