import test from 'node:test';
import assert from 'node:assert';
import http from 'node:http';
import { createServer } from '../src/index.js';
import { Account } from '@stellar/stellar-sdk';

test('server tests', async (t) => {
  let friendbotCalled = 0;
  let friendbotStatus = 200;
  
  const friendbot = http.createServer((req, res) => {
    friendbotCalled++;
    res.writeHead(friendbotStatus);
    res.end('ok');
  });
  
  await new Promise(resolve => friendbot.listen(0, resolve));
  const friendbotUrl = `http://localhost:${friendbot.address().port}/?addr=`;

  let rpcSendStatus = "SUCCESS";
  
  const mockRpc = {
    getAccount: async (pubKey) => {
      return new Account(pubKey, "1");
    },
    prepareTransaction: async (tx) => {
      return tx;
    },
    sendTransaction: async (tx) => {
      if (rpcSendStatus === "ERROR") {
        return { status: "ERROR", errorResultXdr: "mock_error" };
      }
      return { status: "SUCCESS" };
    }
  };

  const app = createServer({
    rpc: mockRpc,
    friendbotUrl,
    contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM'
  });

  const server = app.listen(0);
  const port = server.address().port;
  const baseUrl = `http://localhost:${port}`;

  await t.test('legal config submits init', async () => {
    friendbotCalled = 0;
    friendbotStatus = 200;
    rpcSendStatus = "SUCCESS";
    
    const res = await fetch(`${baseUrl}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ board_width: 3, board_height: 3, win_length: 3, first_player: 'x' })
    });
    
    const data = await res.json();
    if (res.status !== 200) console.log(data);
    assert.strictEqual(res.status, 200);
    assert.ok(data.sessionId);
    assert.strictEqual(data.contractId, 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM');
    assert.strictEqual(friendbotCalled, 1);
  });

  await t.test('illegal config returns 400 and does not call friendbot', async () => {
    friendbotCalled = 0;
    const res = await fetch(`${baseUrl}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ board_width: 2, board_height: 3, win_length: 3, first_player: 'x' })
    });
    
    assert.strictEqual(res.status, 400);
    assert.strictEqual(friendbotCalled, 0);
  });

  await t.test('friendbot failure returns 500', async () => {
    friendbotCalled = 0;
    friendbotStatus = 500;
    
    const res = await fetch(`${baseUrl}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ board_width: 3, board_height: 3, win_length: 3, first_player: 'x' })
    });
    
    assert.strictEqual(res.status, 500);
    assert.strictEqual(friendbotCalled, 1);
  });

  await t.test('rpc failure returns 502', async () => {
    friendbotCalled = 0;
    friendbotStatus = 200;
    rpcSendStatus = "ERROR";
    
    const res = await fetch(`${baseUrl}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ board_width: 3, board_height: 3, win_length: 3, first_player: 'x' })
    });
    
    assert.strictEqual(res.status, 502);
  });

  server.close();
  friendbot.close();
});
