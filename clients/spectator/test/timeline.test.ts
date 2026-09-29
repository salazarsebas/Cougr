import assert from 'node:assert/strict';
import test from 'node:test';

import {
  decodeTurnState,
  formatBoardAscii,
  describeStateTransition,
  ReplayTimeline,
  STATUS_IN_PROGRESS,
  STATUS_X_WINS,
  STATUS_O_WINS,
  STATUS_DRAW,
  CELL_X,
  CELL_O,
  type TimelineFrame,
  type MatchState,
} from '../src/index.ts';

test('decodeTurnState extracts fields from 9-byte buffer correctly', () => {
  const buf = new Uint8Array(9);
  const view = new DataView(buf.buffer);
  buf[0] = 1; // is_x_turn: true
  view.setUint32(1, 4); // move_count: 4
  view.setUint32(5, STATUS_IN_PROGRESS); // status: 0

  const decoded = decodeTurnState(buf);
  assert.equal(decoded.isXTurn, true);
  assert.equal(decoded.moveCount, 4);
  assert.equal(decoded.status, STATUS_IN_PROGRESS);
  assert.equal(decoded.statusLabel, 'in_progress');
});

test('decodeTurnState rejects buffers shorter than 9 bytes', () => {
  assert.throws(
    () => decodeTurnState(new Uint8Array(8)),
    /TurnState byte length must be at least 9/,
  );
});

test('formatBoardAscii formats 3x3 board correctly', () => {
  const cells = [
    CELL_X, 0, 0,
    0, CELL_O, 0,
    0, 0, CELL_X,
  ];
  const rendered = formatBoardAscii(cells);
  assert.equal(
    rendered,
    ' X | . | . \n---+---+---\n . | O | . \n---+---+---\n . | . | X ',
  );
});

test('describeStateTransition detects cell placements and outcomes', () => {
  const prevTurn = { isXTurn: true, moveCount: 0, status: STATUS_IN_PROGRESS, statusLabel: 'in_progress' as const };
  const currTurn = { isXTurn: false, moveCount: 1, status: STATUS_IN_PROGRESS, statusLabel: 'in_progress' as const };
  const prevBoard = [0, 0, 0, 0, 0, 0, 0, 0, 0];
  const currBoard = [1, 0, 0, 0, 0, 0, 0, 0, 0];

  const desc = describeStateTransition(prevTurn, currTurn, prevBoard, currBoard);
  assert.match(desc, /X placed at cell 0/);
  assert.match(desc, /Move #1, next: O/);

  const winTurn = { isXTurn: false, moveCount: 5, status: STATUS_X_WINS, statusLabel: 'x_wins' as const };
  const winDesc = describeStateTransition(currTurn, winTurn, prevBoard, currBoard);
  assert.match(winDesc, /Player X WINS!/);
});

function makeFrame(step: number, status = STATUS_IN_PROGRESS): TimelineFrame {
  const dummyState: MatchState = {
    entityId: 1,
    turnState: { isXTurn: step % 2 === 0, moveCount: step, status, statusLabel: 'in_progress' },
    board: [0, 0, 0, 0, 0, 0, 0, 0, 0],
    components: {},
  };
  return {
    step,
    ledger: 100 + step,
    ledgerClosedAt: '2026-09-29T10:00:00Z',
    txHash: `hash-${step}`,
    pagingToken: `token-${step}`,
    update: {
      family: 'set',
      componentType: 'turnst',
      entityId: 1,
      ledger: 100 + step,
      ledgerClosedAt: '2026-09-29T10:00:00Z',
      txHash: `hash-${step}`,
      pagingToken: `token-${step}`,
      eventId: `event-${step}`,
      raw: {} as any,
      data: new Uint8Array(9),
    },
    action: `Action ${step}`,
    state: dummyState,
  };
}

test('ReplayTimeline stepping and boundary navigation', () => {
  const frames = [makeFrame(0), makeFrame(1), makeFrame(2, STATUS_X_WINS)];
  const timeline = new ReplayTimeline(frames);

  assert.equal(timeline.length, 3);
  assert.equal(timeline.cursorIndex, 0);
  assert.equal(timeline.current()?.step, 0);

  // Step forward
  const f1 = timeline.stepForward();
  assert.equal(f1?.step, 1);
  assert.equal(timeline.cursorIndex, 1);

  const f2 = timeline.stepForward();
  assert.equal(f2?.step, 2);
  assert.equal(timeline.cursorIndex, 2);

  // Cannot step past end
  const pastEnd = timeline.stepForward();
  assert.equal(pastEnd, null);
  assert.equal(timeline.cursorIndex, 2);

  // Step backward
  const b1 = timeline.stepBackward();
  assert.equal(b1?.step, 1);

  // Jump to step
  timeline.goToStep(0);
  assert.equal(timeline.current()?.step, 0);

  // Jump to last
  timeline.last();
  assert.equal(timeline.current()?.step, 2);
  assert.equal(timeline.isFinished(), true);

  const summary = timeline.getSummary();
  assert.equal(summary.totalSteps, 3);
  assert.equal(summary.winner, 'X');
  assert.equal(summary.startLedger, 100);
  assert.equal(summary.endLedger, 102);
});

test('ReplayTimeline formats state log and full log cleanly', () => {
  const frames = [makeFrame(0), makeFrame(1, STATUS_X_WINS)];
  const timeline = new ReplayTimeline(frames);

  const singleLog = timeline.formatStateLog();
  assert.match(singleLog, /=== Step 1\/2 \| Ledger 100/);
  assert.match(singleLog, /Action: Action 0/);
  assert.match(singleLog, /Turn State: moves=0/);

  const fullLog = timeline.formatFullLog();
  assert.match(fullLog, /COUGR SPECTATOR REPLAY TIMELINE/);
  assert.match(fullLog, /Outcome: X Wins!/);
});
