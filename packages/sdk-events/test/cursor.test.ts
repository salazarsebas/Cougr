import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  advanceCursor,
  newEventCursor,
  pageCougrEvents,
  type CougrEventPage,
  type EventCursor,
  type GetEventsFn,
  type GetEventsRequest,
  type GetEventsResponse,
  type SorobanRpcContractEvent,
  type SorobanRpcEventsPage,
} from '../src/index.ts';

interface FixtureFile {
  events: Array<{ kind: string; event: SorobanRpcContractEvent }>;
}

const fixtures = JSON.parse(
  readFileSync(new URL('../fixtures/events.json', import.meta.url), 'utf8'),
) as FixtureFile;

const [setEvent, delEvent, richEvent] = fixtures.events.map((entry) => entry.event);
assert.ok(setEvent && delEvent && richEvent);

function rpcEvent(overrides: Partial<SorobanRpcContractEvent>): SorobanRpcContractEvent {
  return {
    ...(setEvent as SorobanRpcContractEvent),
    ...overrides,
  };
}

function page(
  events: SorobanRpcContractEvent[],
  cursor: string | undefined,
  latestLedger = 120,
): GetEventsResponse {
  const result: SorobanRpcEventsPage = { events, latestLedger };
  if (cursor !== undefined) result.cursor = cursor;
  return result;
}

/** Deterministic fake RPC server that also records the requests it saw. */
function fakeServer(pages: GetEventsResponse[]): { getEvents: GetEventsFn; requests: GetEventsRequest[] } {
  const requests: GetEventsRequest[] = [];
  let index = 0;
  const getEvents: GetEventsFn = async (request) => {
    requests.push({ ...request });
    const next = pages[index];
    index += 1;
    if (next === undefined) return page([], undefined);
    return next;
  };
  return { getEvents, requests };
}

test('advanceCursor prefers the page cursor and keeps the last ledger', () => {
  const cursor = newEventCursor({ ledger: 10 });
  const next = advanceCursor(cursor, page([rpcEvent({ ledger: 42, pagingToken: '42-1' })], 'page-1'));
  assert.equal(next.pagingToken, 'page-1');
  assert.equal(next.ledger, 42);
});

test('advanceCursor falls back to the last event paging token', () => {
  const cursor = newEventCursor({ ledger: 10 });
  const next = advanceCursor(cursor, page([rpcEvent({ ledger: 42, pagingToken: '42-1' })], undefined));
  assert.equal(next.pagingToken, '42-1');
  assert.equal(next.ledger, 42);
});

test('pageCougrEvents decodes pages and can be paused after one page', async () => {
  const first = page(
    [rpcEvent({ ledger: 100, pagingToken: '100-1', id: '100-1' }), rpcEvent({ ledger: 101, pagingToken: '101-1', id: '101-1' })],
    'page-1',
  );
  const second = page([rpcEvent({ ledger: 102, pagingToken: '102-1', id: '102-1' })], 'page-2');
  const { getEvents, requests } = fakeServer([first, second]);

  const iterator = pageCougrEvents(getEvents, { startLedger: 100, maxPages: 2 });
  const firstPage = (await iterator.next()).value as CougrEventPage;
  assert.equal(firstPage.updates.length, 2);
  assert.equal(firstPage.cursor.pagingToken, 'page-1');
  assert.equal(requests[0]?.startLedger, 100);
  assert.equal(requests[0]?.cursor, undefined);

  // Simulate a client that processed page 1 and persisted its cursor.
  const persisted = JSON.parse(JSON.stringify(firstPage.cursor)) as EventCursor;

  // Resume from the persisted cursor with a fresh generator.
  const resumed = pageCougrEvents(getEvents, { cursor: persisted, maxPages: 1 });
  const secondPage = (await resumed.next()).value as CougrEventPage;
  assert.equal(requests[1]?.cursor, 'page-1');
  assert.equal(requests[1]?.startLedger, undefined);
  assert.equal(secondPage.updates.length, 1);
  assert.equal(secondPage.updates[0]?.pagingToken, '102-1');
  assert.equal(secondPage.cursor.pagingToken, 'page-2');
});

test('pageCougrEvents skips paging tokens the client already saw', async () => {
  const repeated = rpcEvent({ ledger: 100, pagingToken: '100-1', id: '100-1' });
  const { getEvents } = fakeServer([
    page([repeated], undefined),
    page([repeated, rpcEvent({ ledger: 101, pagingToken: '101-1', id: '101-1' })], undefined),
  ]);
  const seen = new Set<string>(['100-1']);
  const pages: CougrEventPage[] = [];
  for await (const eventPage of pageCougrEvents(getEvents, { startLedger: 100, maxPages: 2, seenPagingTokens: seen })) {
    pages.push(eventPage);
  }
  assert.equal(pages[0]?.updates.length, 0);
  assert.equal(pages[1]?.updates.length, 1);
  assert.equal(pages[1]?.updates[0]?.pagingToken, '101-1');
});

test('pageCougrEvents stops when the stream stops advancing', async () => {
  const stuck = rpcEvent({ ledger: 100, pagingToken: '100-1', id: '100-1' });
  const { getEvents, requests } = fakeServer([page([stuck], '100-1'), page([stuck], '100-1')]);
  const pages: CougrEventPage[] = [];
  for await (const eventPage of pageCougrEvents(getEvents, { startLedger: 100 })) {
    pages.push(eventPage);
  }
  // Second page yields the same token, so the loop must not spin forever.
  assert.equal(pages.length, 2);
  assert.equal(requests.length, 2);
});

test('pageCougrEvents requires a start ledger or cursor', async () => {
  const { getEvents } = fakeServer([]);
  const iterator = pageCougrEvents(getEvents, {});
  await assert.rejects(async () => {
    await iterator.next();
  }, /startLedger or a resume cursor/);
});
