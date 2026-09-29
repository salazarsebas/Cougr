# cougr-spectator-client

Spectator and replay viewer client for Cougr turn-based game contracts, built directly on **`cougr-sdk-events`**.

This client proves `cougr-sdk-events` as a real downstream consumer rather than an uncalled library. It reads `COUGR` component events (`set`, `del`, `rich`) published by game contracts, reconstructs an ordered match timeline, decodes component changes into domain state (turn, moves, board, outcome), and provides step-by-step replay and spectator navigation.

---

## How it exercises `cougr-sdk-events`

`cougr-spectator-client` imports and consumes the public API surface of `cougr-sdk-events` without reimplementing event decoding or pagination:

| `cougr-sdk-events` export | How `cougr-spectator-client` uses it |
| ------------------------- | ------------------------------------ |
| `decodeCougrEvents(events)` | Decodes raw Soroban RPC contract events into typed `CougrEventUpdate` records (`set`, `del`, `rich`). |
| `decodeCougrPage(page)` | Decodes full RPC event pages while preserving RPC page metadata. |
| `resolveRichComponentUpdate(update, reader)` | Satisfies the `requiresFollowUpRead` compile-time guarantee for `rich` components (e.g. `Board` and `Players`), fetching out-of-band instance storage state. |
| `cougrEventFilter(options)` | Generates Soroban RPC topic filters (`[["COUGR", "set"], ["COUGR", "del"], ["COUGR", "rich"]]`) scoped to the target contract ID. |
| `pageCougrEvents(getEvents, options)` | Asynchronously walks paginated event streams, yielding deduplicated updates, advancing cursors, and stopping when caught up. |
| `advanceCursor`, `newEventCursor` | Maintains streaming state over RPC pages without hand-crafted paging token logic. |
| `serializeCursor`, `parseCursor` | Persists and resumes the spectator stream position across client restarts. |

---

## Target Contract: Turn-Based Game

The spectator targets Cougr's reference turn-based contract (`cli/templates/turn-based` and `examples/tic_tac_toe`):
- **`TurnState` (`"turnst"`)**: Fixed-size component (`is_x_turn: bool`, `move_count: u32`, `status: u32`). Emitted as `set` updates carrying 9 big-endian bytes decoded by `decodeTurnState()`.
- **`Board` (`"board"`)**: Rich component containing the 3×3 cell markers (0 = Empty, 1 = X, 2 = O). Emitted as `rich` updates and resolved via `resolveRichComponentUpdate`.
- **`Players` (`"players"`)**: Rich component containing player addresses.
- **Session Cleanup (`del`)**: Emitted when a match session is removed, unsetting components in the timeline state.

---

## Features

- **Replay Timeline**: Orders decoded updates and produces an immutable `MatchState` snapshot for each frame.
- **Frame Navigation**: Step forward (`stepForward()`), step backward (`stepBackward()`), jump to arbitrary step (`goToStep(n)`), first, last.
- **ASCII State Log**: Renders a clean ASCII view of the 3×3 grid, turn count, current turn owner, and match status at each frame.
- **Spectator Stream**: Consumes live or recorded event streams via `SpectatorClient` and `pageCougrEvents`.
- **Recorded Fixture**: Includes a full 14-event fixture (`fixtures/turn_based_match.json`) capturing a complete match from initialization through a 5-move X victory to session removal.
- **Zero Live Network in Tests**: CI and unit tests run against fixtures and mock RPC servers with no dependency on external endpoints.

---

## Quick Start

### Build and Test

```sh
cd clients/spectator
npm install
npm test
npm run build
```

### Run Replay Viewer

To replay the recorded 5-move winning match:

```sh
npm start
```

Or run step-by-step:

```sh
node dist/cli.js --step-by-step
```

### Example State Log Output

```text
=================================================================
               COUGR SPECTATOR REPLAY TIMELINE                   
=================================================================
Total steps: 14 | Moves: 5 | Status: x_wins
Outcome: X Wins!
Ledger span: 100 -> 106
=================================================================

=== Step 13/14 | Ledger 105 [2026-09-29T10:00:25Z] ===
Action: X placed at cell 2 (row 0, col 2) - Player X WINS!
Event: [rich] component=board entity=1 token=105-02
Players: X = GBALICE... vs O = GBBOB...
Turn State: moves=5, turn=O, status=x_wins (1)
Board:
 X | X | X 
---+---+---
 . | O | O 
---+---+---
 . | . | . 
```

---

## Programmatic API

```ts
import {
  replayFromFixture,
  replayFromEvents,
  SpectatorClient,
} from 'cougr-spectator-client';

// 1. Replay from a fixture
const timeline = await replayFromFixture(fixtureData);
console.log(timeline.formatFullLog());

// Step navigation
timeline.first();
while (timeline.current()) {
  console.log(timeline.formatStateLog());
  if (!timeline.stepForward()) break;
}

// 2. Spectate live via Soroban RPC
const client = new SpectatorClient({
  contractId: 'C...',
  startLedger: 100_000,
});

for await (const page of client.spectate(getEvents)) {
  console.log(`Received ${page.updates.length} updates`);
  console.log(client.timeline.formatStateLog());
  localStorage.setItem('spectator-cursor', client.serializedCursor);
}
```

---

## License

MIT
