# Cougr Studio

The studio sandbox view for a turn-based Cougr match: a 2D board that draws any
legal `TurnBasedConfig`, the deliberate wait while a Soroban transaction
finalizes, and real error states when the RPC or friendbot fails. It is a
testnet sandbox, and the screen says so.

This is the view from the issue "render variable boards and the run latency
state" (`#321`). It is not a general game engine: no sprites, no physics, no
second genre, no custodial keys.

## What it does

- **Any board size.** Geometry comes from `config.board.{width,height}`, so a
  3×3 tic-tac-toe, a 7×6 Connect Four and a 64×64 board all render from the same
  code. Nothing assumes 3×3.
- **A deliberate wait.** `finalizing` is a run phase with its own panel: a
  spinner, the submitted transaction hash, elapsed time, poll count and a
  written explanation that Stellar closes a ledger roughly every five seconds.
  The board keeps drawing the last confirmed state underneath, so the screen is
  never empty.
- **Real errors, never a hang.** RPC failures, friendbot failures and a
  finality timeout all become an `error` phase with a title, an explanation, what
  did not happen and a retry. The poll loop is bounded by `maxPolls`, so a node
  that accepts a transaction and never confirms produces a timeout instead of a
  spinner that never stops.
- **An unmissable sandbox warning.** `SANDBOX_WARNING` is a constant, not a
  prop: every render includes it.

## Read path: poll `get_state`

The choice and its reasoning are in `src/readPath.js` as data
(`readPathJustification()`), so the UI, this README and the pull request cannot
drift apart. In short:

- `RichComponentChangedEvent` carries only `{ component_type, entity_id }`. Its
  own doc comment in `src/ecs_events.rs` says off-chain indexers must query the
  contract for the updated value, so an event subscriber still needs a
  follow-up `get_state` read.
- The `set` event carries raw component bytes, so an event-only board would have
  to reimplement the contract's serialization to rebuild `GameState`.
- `get_state` returns `{ cells, status, is_x_turn, move_count }` in one call —
  the shape the board renders and the shape a live host passes through.
- Polling converges even when a notification is missed; an event subscriber that
  drops one update stays stale.

The trade-off is a bounded number of extra RPC reads during a few-second
finality window, which the ~5s ledger close dominates anyway.

## Using it

The view is framework-free and returns an HTML string, so a host mounts it with
`innerHTML` and a request path is optional:

```js
import { renderStudioView } from './studio/src/view.js';
import { runMatch } from './studio/src/run.js';

element.innerHTML = renderStudioView(model);

// Fixture data and live state are the same shape. A live client is injected,
// which is also what makes the flow testable without a chain.
const model = await runMatch({
  config,                                   // TurnBasedConfig
  previousState,                            // last confirmed GameState
  move: { cell: 4 },
  client: {
    ensureFunded: () => fundWithFriendbot(),    // may reject -> friendbot error
    submitMove: (move) => contract.make_move(move), // may reject -> rpc error
    readState: () => contract.get_state(),      // may reject -> rpc error
  },
  hooks: { onPhase: (next) => { element.innerHTML = renderStudioView(next); } },
});
```

`runMatch` never rejects. Every failure comes back as a model with
`phase: 'error'` and an `error` descriptor, so a host cannot leave a spinner up
by forgetting a `catch`.

## Fixtures

Recorded JSON in `fixtures/` so the view can merge before the on-chain backend
exists:

| Fixture | Board | What it pins |
|---|---|---|
| `3x3-win` | 3×3 | smallest board, win on the top row |
| `3x3-draw` | 3×3 | a full board with no line |
| `7x6-win` | 7×6 | a larger, non-square board with `winLength: 4` |
| `3x3-finalizing` | 3×3 | the finality wait state |
| `3x3-rpc-error` | 3×3 | RPC failure after a submitted move |
| `3x3-friendbot-error` | 3×3 | funding failure before any move |

`tools/validate-fixtures.js` checks that each fixture is a legal
`TurnBasedConfig` with a matching state, and that its declared `expect` block
still agrees with what the board computes — so a renderer change that moves a
cell or a label fails CI instead of silently drifting.

## Running it

The studio has no npm dependencies. It resolves colors from the shared design
tokens package, whose `dist/` is generated rather than committed, so build the
tokens first:

```sh
node ../packages/tokens/build.js          # or: npm run build:tokens
npm test                                  # node --test
npm run validate:fixtures                 # legality + expectation checks
npm run render:fixtures                   # writes out/<id>.html and out/<id>.svg
```

`out/` is gitignored; it is an artifact for a reviewer, not a source file.

## Layout

```
studio/
  src/
    config.js     TurnBasedConfig shape, legality, cell indexing
    state.js      GameState validation, status labels, run phases
    board.js      SVG renderer for any width × height
    readPath.js   polling loop + the justification for choosing it
    run.js        the run state machine and error taxonomy
    view.js       the fragment: board, status, wait, errors, sandbox warning
    theme.js      design tokens, resolved at render time
  fixtures/       recorded JSON scenarios
  test/           node:test suites
  tools/          fixture loader, validator, artifact renderer
```

## CI

`.github/workflows/cougr-studio.yml` is path-scoped to `studio/**`,
`packages/tokens/tokens.json` and the workflow file itself, mirroring
`.github/workflows/cougr-site.yml`. It builds the tokens, validates the
fixtures and runs `node --test`. It does not run the Rust `clippy -D warnings`
gate, and Rust-only changes do not trigger it.
