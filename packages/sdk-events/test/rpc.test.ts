import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  cougrEventFilter,
  createGetEvents,
  type SorobanRpcContractEvent,
} from '../src/index.ts';

interface FixtureFile {
  events: Array<{ kind: string; event: SorobanRpcContractEvent }>;
}

const fixtures = JSON.parse(
  readFileSync(new URL('../fixtures/events.json', import.meta.url), 'utf8'),
) as FixtureFile;
const setEvent = fixtures.events[0]?.event;
assert.ok(setEvent);

function capture() {
  const calls: Array<{ url: string; body: unknown }> = [];
  const fetchLike = async (url: string, init: { body: string }) => {
    calls.push({ url, body: JSON.parse(init.body) as unknown });
    return {
      ok: true,
      status: 200,
      async json() {
        return { jsonrpc: '2.0', id: 1, result: { events: [setEvent], latestLedger: 500 } };
      },
      async text() {
        return '';
      },
    };
  };
  return { calls, fetchLike };
}

test('createGetEvents posts a getEvents JSON-RPC call', async () => {
  const { calls, fetchLike } = capture();
  const getEvents = createGetEvents({ url: 'https://rpc.example', fetch: fetchLike });
  const result = await getEvents({ startLedger: 400, filters: [cougrEventFilter()], limit: 50 });

  assert.equal(result.latestLedger, 500);
  assert.equal(result.events.length, 1);

  const call = calls[0];
  assert.ok(call);
  assert.equal(call.url, 'https://rpc.example');
  const body = call.body as {
    method: string;
    params: Array<Record<string, unknown>>;
  };
  assert.equal(body.method, 'getEvents');
  assert.equal(body.params[0]?.['startLedger'], 400);
  assert.equal(body.params[0]?.['limit'], 50);
  const filters = body.params[0]?.['filters'] as Array<{ topics: string[][] }>;
  assert.equal(filters[0]?.topics.length, 3);
});

test('a resume cursor replaces startLedger in the request', async () => {
  const { calls, fetchLike } = capture();
  const getEvents = createGetEvents({ url: 'https://rpc.example', fetch: fetchLike });
  await getEvents({ cursor: 'page-9', startLedger: 400 });
  const body = calls[0]?.body as { params: Array<Record<string, unknown>> };
  assert.equal(body.params[0]?.['cursor'], 'page-9');
});

test('RPC errors surface with the JSON-RPC message', async () => {
  const getEvents = createGetEvents({
    url: 'https://rpc.example',
    fetch: async () => ({
      ok: true,
      status: 200,
      async json() {
        return { jsonrpc: '2.0', id: 1, error: { code: -32602, message: 'invalid cursor' } };
      },
      async text() {
        return '';
      },
    }),
  });
  await assert.rejects(() => getEvents({ startLedger: 1 }), /invalid cursor/);
});

test('HTTP failures surface with the status code', async () => {
  const getEvents = createGetEvents({
    url: 'https://rpc.example',
    fetch: async () => ({
      ok: false,
      status: 503,
      async json() {
        return {};
      },
      async text() {
        return 'unavailable';
      },
    }),
  });
  await assert.rejects(() => getEvents({ startLedger: 1 }), /HTTP 503/);
});
