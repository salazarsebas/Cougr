# cougr-sdk-events

Typed decoder for the `COUGR` component events that Cougr contracts publish,
built for the Soroban RPC `getEvents` method.

`src/ecs_events.rs` emits three families under the `COUGR` namespace:

| Topics                            | Emitted by                  | Event data             |
| --------------------------------- | --------------------------- | ---------------------- |
| `("COUGR", "set", <component>)`   | `World::set_typed_observed` | `{ entity_id, data }`  |
| `("COUGR", "del", <component>)`   | `World::remove_observed`    | `{ entity_id }`        |
| `("COUGR", "rich", <component>)`  | `World::set_rich_observed`  | `{ entity_id }` only   |

The SDK turns an RPC event page into typed `set`, `del`, and `rich` records,
gives you topic filters for the three families, and hands you a cursor you can
persist and resume from.

## Install

```sh
npm install cougr-sdk-events
```

The package has **no runtime dependencies**. It ships the small XDR reader it
needs, so there is no `@stellar/stellar-sdk` in your bundle unless you already
use it.

## Quick start

```ts
import {
  cougrEventFilter,
  createGetEvents,
  pageCougrEvents,
  parseCursor,
} from 'cougr-sdk-events';

const rpcUrl = 'https://soroban-testnet.stellar.org';
const contractId = 'C...YOUR_CONTRACT';
const getEvents = createGetEvents({ url: rpcUrl });

const cursor = parseCursor(localStorage.getItem('cougr-cursor') ?? 'null');
const filters = [cougrEventFilter({ contractIds: [contractId] })];

for await (const page of pageCougrEvents(getEvents, {
  filters,
  startLedger: 1_000_000,
  cursor,               // resume where the last run stopped
})) {
  for (const update of page.updates) {
    switch (update.family) {
      case 'set':
        // update.data is the serialized component bytes; decode with the
        // component's own field layout.
        applyComponent(update.entityId, update.componentType, update.data);
        break;
      case 'del':
        removeComponent(update.entityId, update.componentType);
        break;
      case 'rich':
        // update.requiresFollowUpRead === true and there is no `data` field.
        // See "Why a rich event needs a follow-up read" below.
        scheduleRichRead(update);
        break;
    }
  }
  localStorage.setItem('cougr-cursor', JSON.stringify(page.cursor));
}
```

`page.cursor` is `{ pagingToken, ledger }`. Persist it after every page; the
next run resumes with the RPC `cursor` parameter.

## Topic filters

`cougrEventFilter()` builds the three-family filter for you:

```ts
cougrEventFilter({ contractIds: ['C...'] });
// { type: 'contract', contractIds: ['C...'], topics: [
//     [COUGR, set],
//     [COUGR, del],
//     [COUGR, rich],
// ] }
```

Topic positions are ANDed and the topic arrays are ORed, so this subscribes to
exactly the three Cougr families for the given contracts. Narrow to a single
component by adding its symbol as a third, ANDed position:

```ts
cougrEventFilter({ contractIds: ['C...'], componentType: 'position', families: ['set'] });
```

`eventMatchesCougrTopics(event, options)` mirrors the filter client-side, which
is handy when a page was fetched with a wildcard.

## Why a rich event needs a follow-up read

Rich components use Soroban's XDR codec and are stored in the contract's
**instance storage**, not in the `Map`-backed ECS storage. The XDR value is not
embedded in the event — the event only says "this component changed on this
entity". Everything else in `getEvents` (the ledger, the tx hash, the
`(COUGR, rich, component)` topics) is a notification, not a payload.

That is deliberate, and the SDK makes it impossible to ignore:
`RichComponentChangedUpdate.requiresFollowUpRead` is the literal type `true` and
the record has no `data` field, so a client cannot pretend the value was in the
log. Complete the record with `resolveRichComponentUpdate` and a reader that
performs the actual read:

```ts
import { resolveRichComponentUpdate } from 'cougr-sdk-events';

case 'rich':
  const { value } = await resolveRichComponentUpdate(update, ({ entityId }) =>
    contract.get_player_profile(entityId), // your read system / getLedgerEntries
  );
  break;
```

The instance-storage key is `(cougr_rc, entity_id, component_type)`, which is an
implementation detail, so the SDK asks you for the read rather than guessing the
key.

## getEvents recipe

The SDK is a thin layer over one RPC method. You can reproduce a page with curl:

```sh
curl -s https://soroban-testnet.stellar.org \
  -H 'content-type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getEvents",
    "params": [{
      "startLedger": 1000000,
      "filters": [{
        "type": "contract",
        "contractIds": ["C..."],
        "topics": [
          ["AAAADwAAAAdDT1VHUgAAAA==", "AAAADwAAAANzZXQAAAA="],
          ["AAAADwAAAAdDT1VHUgAAAA==", "AAAADwAAAANkZWwAAAA="],
          ["AAAADwAAAAdDT1VHUgAAAA==", "AAAADwAAAARyaWNoAAA="]
        ]
      }],
      "limit": 100
    }]
  }'
```

Each event's `topic` entries and `value` are base64 XDR `ScVal` unions. The
response also carries `latestLedger` and, when available, `cursor`. Pass that
`cursor` back as the request's `cursor` parameter to resume exactly where the
previous page ended.

## Fixtures

`fixtures/events.json` holds one captured event per family. It is generated by
the Rust helper crate in `fixtures/`, which publishes a `set`, a `del`, and a
`rich` event through the real `cougr-core` event types on an
`Env::default()` — no network, no scraping:

```sh
npm run fixtures           # rewrites fixtures/events.json
cargo test --manifest-path fixtures/Cargo.toml   # verifies it is not stale
```

The generator depends on `cougr-core` by path, so the fixtures track the event
schema in `src/ecs_events.rs`. The decoder tests in `test/` consume only the
committed JSON.

## CI

`.github/workflows/sdk-events.yml` is path-scoped to `packages/sdk-events/**`
and this workflow file. It runs the TypeScript typecheck, the decoder tests, and
the build. It does not run the Rust `clippy -D warnings` gate, so Rust-only
changes never trigger it.

## License

MIT
