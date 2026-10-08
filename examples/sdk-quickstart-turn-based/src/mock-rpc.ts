/**
 * Mock RPC factory for the sdk-quickstart-turn-based CI run.
 *
 * Mirrors the pattern in `packages/sdk-events/examples/walkthrough.ts`:
 * a fake `GetEventsFn` backed by the fixture JSON file so the entire
 * integration test runs without any live Soroban node.
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import type {
  GetEventsFn,
  GetEventsRequest,
  SorobanRpcContractEvent,
  SorobanRpcEventsPage,
} from 'cougr-sdk-events';

// ── Load fixture data ─────────────────────────────────────────────────────

const __dirname = dirname(fileURLToPath(import.meta.url));

interface FixtureFile {
  contractId: string;
  startLedger: number;
  events: Array<{ description: string; component: string; event: SorobanRpcContractEvent }>;
}

function loadFixtures(): FixtureFile {
  const fixturePath = join(__dirname, '..', 'fixtures', 'turn-based-events.json');
  return JSON.parse(readFileSync(fixturePath, 'utf8')) as FixtureFile;
}

export const FIXTURE_DATA = loadFixtures();
export const FIXTURE_CONTRACT_ID = FIXTURE_DATA.contractId;
export const FIXTURE_START_LEDGER = FIXTURE_DATA.startLedger;

// ── Mock GetEventsFn ──────────────────────────────────────────────────────

/**
 * Single-page mock: returns all fixture events in one page, then returns
 * empty pages. Suitable for the quickstart which processes one ledger's worth
 * of events from a single `make_move` call.
 */
export function makeMockRpc(): GetEventsFn {
  const allEvents = FIXTURE_DATA.events.map((e) => e.event);
  let served = false;

  return async (_req: GetEventsRequest): Promise<SorobanRpcEventsPage> => {
    if (!served) {
      served = true;
      return { events: allEvents, latestLedger: FIXTURE_START_LEDGER + 1 };
    }
    return { events: [], latestLedger: FIXTURE_START_LEDGER + 1 };
  };
}
