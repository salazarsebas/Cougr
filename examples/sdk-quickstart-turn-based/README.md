# sdk-quickstart-turn-based

A runnable Node/TypeScript script that proves `sdk-core`, `sdk-session`, and
`sdk-events` compose end-to-end against the `turn-based` contract template.

This is **not** a browser app or a deployed client — it is a Node-side script
with no wallet and no UI. Its only job is to demonstrate that a client can:

1. simulate and submit a session-signed `make_move` call via **sdk-core**,
2. scope that call through a **sdk-session**-built `SessionPolicy`, and
3. reconstruct the post-move `GameState` from the decoded COUGR events emitted
   by **sdk-events** — without making a second `get_state` poll.

See [issue #367](https://github.com/salazarsebas/Cougr/issues/367) for the
full scope and definition of done.

---

## Package composition

| Concern | Owner |
|---|---|
| Simulate, sign, submit | **sdk-core** (`TurnBasedClient`) |
| Session keypair and auth entries | **sdk-session** (`SessionBuilder`, `buildSessionAuth`) |
| Event decoding and cursor management | **sdk-events** (`pageCougrEvents`, `decodeCougrEvent`, `resolveRichComponentUpdate`) |
| Business logic (decode bytes, diff state) | This quickstart (`src/game-state.ts`) |

### Signer ownership

- **sdk-core** owns transport only. It attaches auth entries it receives from
  the caller; it never holds key material.
- **sdk-session** owns the session keypair. It derives an ephemeral key from
  `SessionPolicy.sessionKeySeed`, signs the invocation description, and
  returns base64-encoded `SorobanAuthorizationEntry` XDR that sdk-core
  attaches to the assembled transaction.
- **sdk-events** has no signer at all. It is a pure decoder over the Soroban
  `getEvents` RPC response.

---

## Quick start

```bash
# Install the real sdk-events dependency first
cd packages/sdk-events && npm ci && npm run build && cd -

# Install and run
cd examples/sdk-quickstart-turn-based
npm install
node src/quickstart.ts
```

No testnet access is needed. The script runs in fixture mode by default
(`FIXTURE_MODE=true`). To target a real testnet deployment set
`FIXTURE_MODE=false` and configure a deployed contract.

---

## State reconstruction from events

The key claim of this quickstart: after `make_move`, the client **never calls
`get_state` again** to learn the new turn position.

The turn-based contract emits two COUGR events on every move:

| Topics | Event family | Payload |
|---|---|---|
| `("COUGR", "set", "turnst")` | `set` | Full binary `TurnState` (9 bytes) |
| `("COUGR", "rich", "board")` | `rich` | Entity ID only — follow-up read required |

### TurnState: set event → binary decode

`impl_component!(TurnState, "turnst", Table, { is_x_turn: bool, move_count: u32, status: u32 })`
stores a fixed 9-byte blob:

```
byte 0:    bool   — is_x_turn  (1 = true)
bytes 1–4: u32 BE — move_count
bytes 5–8: u32 BE — status     (0=in_progress 1=x_wins 2=o_wins 3=draw)
```

The `data` field on a `ComponentSetUpdate` carries these bytes verbatim.
`decodeTurnState(data)` decodes them without any RPC call.

### Board: rich event → required follow-up read

`impl_rich_component!(Board, "board")` uses Soroban's XDR codec, so the new
value is not carried in the event. sdk-events models this with a distinct type:

```ts
interface RichComponentChangedUpdate {
  family: 'rich';
  requiresFollowUpRead: true;   // literal type — compiler-enforced
  followUp: { kind: 'contractData'; componentType: string; entityId: number };
}
```

There is no `.data` field: the compiler prevents accidental reads. Use
`resolveRichComponentUpdate(update, reader)` to fetch the new value:

```ts
const { value } = await resolveRichComponentUpdate(richUpdate, async ({ componentType, entityId }) => {
  // call getLedgerEntries with (cougr_rc, entityId, componentType) in production
  return getBoard(contractId, entityId);
});
```

---

## SDK stubs

`sdk-core` (#324 / PR #350) and `sdk-session` (#325 / PR #351) are not yet
published. This quickstart ships typed stubs:

| File | Replaces |
|---|---|
| `src/sdk-core-stub.ts` | `cougr-sdk-core` |
| `src/sdk-session-stub.ts` | `cougr-sdk-session` |

The stub exports match the interfaces each issue commits to. Once those PRs
merge, replace the local imports with `from 'cougr-sdk-core'` and
`from 'cougr-sdk-session'` — no other changes needed.

**sdk-events** is the real published package, used directly.

---

## Running tests

```bash
npm test
```

All tests run against fixture data; no testnet connection is required. The
`FIXTURE_MODE` environment variable is checked by `src/quickstart.ts`; the
test suite always runs in fixture mode regardless of that flag.

---

## Directory layout

```
examples/sdk-quickstart-turn-based/
├── fixtures/
│   └── turn-based-events.json   # Mock COUGR events from a single make_move
├── src/
│   ├── sdk-core-stub.ts         # sdk-core interface stub (replace with real pkg)
│   ├── sdk-session-stub.ts      # sdk-session interface stub (replace with real pkg)
│   ├── mock-rpc.ts              # GetEventsFn backed by fixture JSON (CI / no-node)
│   ├── game-state.ts            # TurnState decoder + GameState reconstruction
│   └── quickstart.ts            # End-to-end entry point
└── test/
    └── quickstart.test.ts       # node:test suite for all three packages
```

---

## CI

[`.github/workflows/sdk-quickstart-turn-based.yml`](../../.github/workflows/sdk-quickstart-turn-based.yml)
runs on any change under `examples/sdk-quickstart-turn-based/` or
`packages/sdk-events/`. It:

1. Builds `packages/sdk-events` (the only real dependency).
2. Installs this directory's dependencies.
3. Typechecks.
4. Runs `npm test` with `FIXTURE_MODE=true`.
5. Smoke-runs `node src/quickstart.ts` to verify the script exits cleanly.

No other CI workflow triggers for a change limited to this path.
