import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  COUGR_NAMESPACE,
  buildCougrTopicFilters,
  decodeCougrEvent,
  decodeCougrEvents,
  decodeCougrTopics,
  eventMatchesCougrTopics,
  isCougrEvent,
  isRichComponentChanged,
  type CougrEventFamily,
  type CougrEventUpdate,
  type RichComponentChangedUpdate,
  type SorobanRpcContractEvent,
  serializeCursor,
  parseCursor,
  newEventCursor,
} from '../src/index.ts';

interface FixtureEvent {
  kind: CougrEventFamily;
  event: SorobanRpcContractEvent;
}

interface FixtureFile {
  generatedBy: string;
  events: FixtureEvent[];
}

const fixtures = JSON.parse(
  readFileSync(new URL('../fixtures/events.json', import.meta.url), 'utf8'),
) as FixtureFile;

function fixture(kind: CougrEventFamily): FixtureEvent {
  const found = fixtures.events.find((entry) => entry.kind === kind);
  assert.ok(found, `fixture for ${kind} is missing`);
  return found;
}

function decodeFixture(kind: CougrEventFamily): CougrEventUpdate {
  const update = decodeCougrEvent(fixture(kind).event);
  assert.ok(update, `fixture ${kind} did not decode`);
  return update;
}

test('fixtures cover set, del, and rich', () => {
  const kinds = new Set(fixtures.events.map((entry) => entry.kind));
  assert.deepEqual([...kinds].sort(), ['del', 'rich', 'set']);
  assert.match(fixtures.generatedBy, /cougr-core/);
});

test('set fixture decodes to typed bytes plus entity id', () => {
  const update = decodeFixture('set');
  assert.equal(update.family, 'set');
  assert.equal(update.componentType, 'position');
  assert.equal(update.entityId, 7);
  assert.ok(update.family === 'set' && update.data instanceof Uint8Array);
  if (update.family === 'set') {
    assert.deepEqual([...update.data], [1, 2, 3, 4]);
  }
});

test('del fixture decodes without a payload', () => {
  const update = decodeFixture('del');
  assert.equal(update.family, 'del');
  assert.equal(update.componentType, 'position');
  assert.equal(update.entityId, 7);
  assert.equal('data' in update, false);
});

test('rich fixture forces a follow-up read and carries no value', () => {
  const update = decodeFixture('rich');
  assert.equal(update.family, 'rich');
  assert.ok(isRichComponentChanged(update));
  assert.equal(update.requiresFollowUpRead, true);
  assert.equal('data' in update, false);
  assert.deepEqual(update.followUp, {
    kind: 'contractData',
    contractId: update.contractId,
    componentType: 'player_profile',
    entityId: 7,
  });
  // The type-level guarantee is the point: a consumer has to acknowledge the
  // missing payload before it can use the record.
  const read = (update as RichComponentChangedUpdate).requiresFollowUpRead satisfies true;
  assert.equal(read, true);
});

test('decodeCougrEvents drops non-Cougr events', () => {
  const updates = decodeCougrEvents(fixtures.events.map((entry) => entry.event));
  assert.equal(updates.length, fixtures.events.length);
});

test('isCougrEvent rejects unrelated topics', () => {
  const unrelated: SorobanRpcContractEvent = {
    ...fixture('del').event,
    topic: [fixture('del').event.topic[0] as string, 'AAAAAA=='],
  };
  assert.equal(isCougrEvent(unrelated), false);
  assert.equal(decodeCougrEvent(unrelated), null);
  assert.equal(decodeCougrTopics(unrelated), null);
});

test('topic filters ask for exactly the three families', () => {
  const filters = buildCougrTopicFilters();
  assert.equal(filters.length, 3);
  const families = filters.map((filter) => filter[1]);
  assert.deepEqual(families, [
    fixtures.events[0]?.event.topic[1],
    fixtures.events[1]?.event.topic[1],
    fixtures.events[2]?.event.topic[1],
  ]);
  assert.ok(filters.every((filter) => filter[0] === filters[0]?.[0]));
  assert.equal(COUGR_NAMESPACE, 'COUGR');
});

test('component-scoped filter adds a third ANDed topic', () => {
  const [filter] = buildCougrTopicFilters({ componentType: 'position', families: ['set'] });
  assert.ok(filter);
  assert.equal(filter.length, 3);
  const decoded = decodeCougrTopics({ ...fixture('set').event, topic: filter });
  assert.equal(decoded?.componentType, 'position');
});

test('client-side matcher mirrors the RPC filter', () => {
  const setEvent = fixture('set').event;
  assert.equal(eventMatchesCougrTopics(setEvent), true);
  assert.equal(eventMatchesCougrTopics(setEvent, { families: ['del'] }), false);
  assert.equal(eventMatchesCougrTopics(setEvent, { componentType: 'position' }), true);
  assert.equal(eventMatchesCougrTopics(setEvent, { componentType: 'health' }), false);
});

test('cursor JSON round-trips', () => {
  const cursor = newEventCursor({ pagingToken: '100-1', ledger: 100 });
  assert.deepEqual(parseCursor(serializeCursor(cursor)), cursor);
  assert.throws(() => parseCursor('"nope"'));
});
