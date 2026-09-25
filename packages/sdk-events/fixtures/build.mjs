// Turns the `kind / topic / value` lines from
// `fixtures/src/main.rs` into the committed RPC fixtures.
//
//   cargo run --manifest-path fixtures/Cargo.toml | node fixtures/build.mjs > fixtures/events.json
//
// The event metadata below (ledger, tx hash, ...) is fixed so the output is
// deterministic; only topic/value XDR comes from the Rust run.

import { readFileSync } from 'node:fs';

const CONTRACT_ID = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
const TX_HASH = 'f'.repeat(64);
const LEDGER = 100;
const LEDGER_CLOSED_AT = '2026-09-24T20:38:03Z';
const KINDS = new Set(['set', 'del', 'rich']);

const input = readFileSync(0, 'utf8');
const events = [];
let current = null;

for (const raw of input.split('\n')) {
  const line = raw.trim();
  if (line === '') continue;
  if (KINDS.has(line)) {
    current = { kind: line, topics: [], value: null };
    events.push(current);
    continue;
  }
  if (current === null) throw new Error(`topic/value before any kind: ${line}`);
  if (line.startsWith('topic ')) {
    current.topics.push(line.slice('topic '.length));
    continue;
  }
  if (line.startsWith('value ')) {
    current.value = line.slice('value '.length);
    continue;
  }
  throw new Error(`unexpected generator line: ${line}`);
}

for (const event of events) {
  if (event.topics.length !== 3) {
    throw new Error(`expected 3 topics for ${event.kind}, got ${event.topics.length}`);
  }
  if (event.value === null) throw new Error(`missing value for ${event.kind}`);
}

const fixtures = {
  generatedBy:
    'packages/sdk-events/fixtures (cougr-core events on soroban-sdk Env::default(), no network)',
  events: events.map(({ kind, topics, value }) => ({
    kind,
    event: {
      type: 'contract',
      ledger: LEDGER,
      ledgerClosedAt: LEDGER_CLOSED_AT,
      contractId: CONTRACT_ID,
      id: `${LEDGER}-${kind}`,
      pagingToken: `${LEDGER}-${kind}`,
      inSuccessfulContractCall: true,
      txHash: TX_HASH,
      topic: topics,
      value,
    },
  })),
};

process.stdout.write(`${JSON.stringify(fixtures, null, 2)}\n`);
