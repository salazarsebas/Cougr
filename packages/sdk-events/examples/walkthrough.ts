/**
 * End-to-end walkthrough of sdk-events using the committed fixture data.
 *
 * This file is the source of truth for the patterns shown in the README.
 * Run it directly or let test/example.test.ts drive the assertions.
 */

import { readFileSync } from 'node:fs';

import {
  cougrEventFilter,
  pageCougrEvents,
  resolveRichComponentUpdate,
  isRichComponentChanged,
  serializeCursor,
  parseCursor,
  type CougrEventUpdate,
  type GetEventsFn,
  type GetEventsRequest,
  type SorobanRpcContractEvent,
  type SorobanRpcEventsPage,
} from '../src/index.ts';

// ── fixtures ─────────────────────────────────────────────────────────────────

interface FixtureFile {
  events: Array<{ kind: string; event: SorobanRpcContractEvent }>;
}

const fixtures = JSON.parse(
  readFileSync(new URL('../fixtures/events.json', import.meta.url), 'utf8'),
) as FixtureFile;

// ── step 1: build a COUGR topic filter ───────────────────────────────────────

export function buildFilter(contractId: string) {
  return cougrEventFilter({ contractIds: [contractId] });
}

// ── step 2: fake RPC server backed by fixture events ─────────────────────────

/**
 * Two-page fake: page 1 has set+del, page 2 has rich.
 * The cursor on page 1 tells the generator to ask for page 2 next.
 */
export function makeFixtureServer(): GetEventsFn {
  const allEvents = fixtures.events.map((e) => e.event);
  const pages: SorobanRpcEventsPage[] = [
    { events: allEvents.slice(0, 2), latestLedger: 100, cursor: '100-del' },
    { events: allEvents.slice(2),    latestLedger: 100 },
  ];
  let index = 0;
  return async (_req: GetEventsRequest) => {
    const page = pages[index] ?? { events: [], latestLedger: 100 };
    index += 1;
    return page;
  };
}

// ── step 3: page through events, persist/resume cursor ───────────────────────

export interface WalkthroughResult {
  page1Updates: CougrEventUpdate[];
  resumedCursor: string;           // serialized cursor after page 1
  page2Updates: CougrEventUpdate[];
  richResolved: { componentType: string; entityId: number; value: unknown };
}

export async function runWalkthrough(): Promise<WalkthroughResult> {
  const contractId = fixtures.events[0]!.event.contractId!;
  const filter = buildFilter(contractId);
  const getEvents = makeFixtureServer();

  // ── page 1 ────────────────────────────────────────────────────────────────
  const gen = pageCougrEvents(getEvents, {
    filters: [filter],
    startLedger: 100,
    maxPages: 1,
  });

  const { value: firstPage } = await gen.next();
  if (!firstPage) throw new Error('expected a first page');

  const page1Updates = firstPage.updates;
  const resumedCursor = serializeCursor(firstPage.cursor);

  // ── page 2: resume from persisted cursor ──────────────────────────────────
  const gen2 = pageCougrEvents(getEvents, {
    filters: [filter],
    cursor: parseCursor(resumedCursor),
    maxPages: 1,
  });

  const { value: secondPage } = await gen2.next();
  if (!secondPage) throw new Error('expected a second page');

  const page2Updates = secondPage.updates;

  // ── rich follow-up read ───────────────────────────────────────────────────
  const richUpdate = page2Updates.find(isRichComponentChanged);
  if (!richUpdate) throw new Error('expected a rich update on page 2');

  // The SDK forces you to acknowledge the missing payload before you can use
  // the record — requiresFollowUpRead is the literal type `true`.
  const { update, value } = await resolveRichComponentUpdate(
    richUpdate,
    ({ componentType, entityId }) =>
      // In production this would call getLedgerEntries or your read system.
      // Here we return a fake value that a real reader would supply.
      Promise.resolve({ level: 42, handle: `${componentType}:${entityId}` }),
  );

  return {
    page1Updates,
    resumedCursor,
    page2Updates,
    richResolved: {
      componentType: update.followUp.componentType,
      entityId: update.followUp.entityId,
      value,
    },
  };
}
