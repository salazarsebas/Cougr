import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import type {
  GetEventsFn,
  GetEventsRequest,
  GetEventsResponse,
  SorobanRpcContractEvent,
} from 'cougr-sdk-events';

import {
  SpectatorClient,
  type MatchFixtureFile,
} from '../src/index.ts';

const fixtureJson = JSON.parse(
  readFileSync(new URL('../fixtures/turn_based_match.json', import.meta.url), 'utf8'),
) as MatchFixtureFile;

/**
 * Deterministic mock RPC server that divides fixture events across pages.
 */
function createMockGetEvents(
  pages: Array<{ events: SorobanRpcContractEvent[]; cursor?: string; latestLedger: number }>,
): { getEvents: GetEventsFn; requests: GetEventsRequest[] } {
  const requests: GetEventsRequest[] = [];
  let pageIndex = 0;

  const getEvents: GetEventsFn = async (request) => {
    requests.push({ ...request });
    const current = pages[pageIndex];
    pageIndex += 1;
    if (!current) {
      return { events: [], latestLedger: 200 };
    }
    return current;
  };

  return { getEvents, requests };
}

test('SpectatorClient pages through match events using pageCougrEvents', async () => {
  const page1 = {
    events: fixtureJson.events.slice(0, 3), // Ledger 100
    cursor: 'page-1',
    latestLedger: 100,
  };
  const page2 = {
    events: fixtureJson.events.slice(3, 7), // Ledgers 101-102
    cursor: 'page-2',
    latestLedger: 102,
  };
  const page3 = {
    events: fixtureJson.events.slice(7), // Ledgers 103-106
    cursor: 'page-3',
    latestLedger: 106,
  };

  const { getEvents, requests } = createMockGetEvents([page1, page2, page3]);

  const client = new SpectatorClient({
    contractId: fixtureJson.metadata?.contractId,
    startLedger: 100,
  });

  const seenPages: string[] = [];
  for await (const page of client.spectate(getEvents)) {
    if (page.updates.length > 0 && page.cursor.pagingToken) {
      seenPages.push(page.cursor.pagingToken);
    }
  }

  assert.deepEqual(seenPages, ['page-1', 'page-2', 'page-3']);
  assert.equal(requests.length, 4); // 3 pages with events + 1 empty terminal page

  // Filters were constructed with cougrEventFilter
  assert.equal(requests[0]?.filters?.[0]?.type, 'contract');
  assert.deepEqual(requests[0]?.filters?.[0]?.contractIds, [fixtureJson.metadata?.contractId]);

  // Timeline accumulated all 14 frames
  assert.equal(client.timeline.length, 14);
  assert.equal(client.cursor.pagingToken, 'page-3');
});

test('SpectatorClient can resume from serialized cursor string', async () => {
  const page1 = {
    events: fixtureJson.events.slice(0, 3),
    cursor: 'page-1',
    latestLedger: 100,
  };
  const page2 = {
    events: fixtureJson.events.slice(3, 7),
    cursor: 'page-2',
    latestLedger: 102,
  };

  const { getEvents, requests } = createMockGetEvents([page1, page2]);

  // First session: consume 1 page and persist cursor
  const client1 = new SpectatorClient({ startLedger: 100 });
  const iterator = client1.spectate(getEvents, { maxPages: 1 });
  await iterator.next();

  const savedCursor = client1.serializedCursor;
  assert.ok(savedCursor.includes('page-1'));

  // Second session: resume with saved cursor
  const client2 = new SpectatorClient({ cursor: savedCursor });
  assert.equal(client2.cursor.pagingToken, 'page-1');

  for await (const _ of client2.spectate(getEvents, { maxPages: 1 })) {
    // page 2
  }

  assert.equal(requests[1]?.cursor, 'page-1');
  assert.equal(client2.timeline.length, 4);
});
