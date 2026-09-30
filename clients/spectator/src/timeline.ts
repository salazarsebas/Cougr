import type { CougrEventUpdate } from 'cougr-sdk-events';
import {
  formatBoardAscii,
  statusToLabel,
  type PlayersState,
  type TurnState,
  type TurnStatus,
  STATUS_X_WINS,
  STATUS_O_WINS,
  STATUS_DRAW,
} from './turn_based.ts';

/**
 * Snapshot of the match's complete state at a single point in the timeline.
 */
export interface MatchState {
  entityId: number;
  turnState?: TurnState;
  board?: number[];
  players?: PlayersState;
  /** Generic bag for any other components attached to this entity. */
  components: Record<string, unknown>;
}

/**
 * A discrete step in the match timeline corresponding to a decoded Cougr event.
 */
export interface TimelineFrame {
  step: number;
  ledger: number;
  ledgerClosedAt: string;
  txHash: string;
  pagingToken: string;
  update: CougrEventUpdate;
  action: string;
  state: MatchState;
}

export interface MatchSummary {
  totalSteps: number;
  moveCount: number;
  status: TurnStatus;
  winner?: 'X' | 'O' | 'Draw';
  startLedger: number;
  endLedger: number;
  playerX?: string;
  playerO?: string;
}

/**
 * Navigation and state inspector for an ordered sequence of match updates.
 */
export class ReplayTimeline {
  readonly #frames: TimelineFrame[];
  #cursorIndex = 0;

  constructor(frames: TimelineFrame[] = []) {
    this.#frames = frames;
    this.#cursorIndex = frames.length > 0 ? 0 : -1;
  }

  get frames(): readonly TimelineFrame[] {
    return this.#frames;
  }

  get length(): number {
    return this.#frames.length;
  }

  get cursorIndex(): number {
    return this.#cursorIndex;
  }

  current(): TimelineFrame | null {
    if (this.#cursorIndex < 0 || this.#cursorIndex >= this.#frames.length) {
      return null;
    }
    return this.#frames[this.#cursorIndex] ?? null;
  }

  stepForward(): TimelineFrame | null {
    if (this.#cursorIndex + 1 < this.#frames.length) {
      this.#cursorIndex += 1;
      return this.#frames[this.#cursorIndex] ?? null;
    }
    return null;
  }

  stepBackward(): TimelineFrame | null {
    if (this.#cursorIndex > 0) {
      this.#cursorIndex -= 1;
      return this.#frames[this.#cursorIndex] ?? null;
    }
    return null;
  }

  goToStep(index: number): TimelineFrame | null {
    if (index >= 0 && index < this.#frames.length) {
      this.#cursorIndex = index;
      return this.#frames[this.#cursorIndex] ?? null;
    }
    return null;
  }

  first(): TimelineFrame | null {
    return this.goToStep(0);
  }

  last(): TimelineFrame | null {
    if (this.#frames.length === 0) return null;
    return this.goToStep(this.#frames.length - 1);
  }

  isFinished(): boolean {
    for (let i = this.#frames.length - 1; i >= 0; i -= 1) {
      const status = this.#frames[i]?.state.turnState?.status;
      if (status !== undefined) {
        return status === STATUS_X_WINS || status === STATUS_O_WINS || status === STATUS_DRAW;
      }
    }
    return false;
  }

  getSummary(): MatchSummary {
    const first = this.#frames[0];
    const last = this.#frames[this.#frames.length - 1];

    // Find latest known turnState (in case trailing frame is a del/cleanup event)
    let turnState = last?.state.turnState;
    if (!turnState) {
      for (let i = this.#frames.length - 1; i >= 0; i -= 1) {
        if (this.#frames[i]?.state.turnState) {
          turnState = this.#frames[i]!.state.turnState;
          break;
        }
      }
    }

    const players = last?.state.players ?? first?.state.players;

    let winner: 'X' | 'O' | 'Draw' | undefined;
    if (turnState?.status === STATUS_X_WINS) winner = 'X';
    else if (turnState?.status === STATUS_O_WINS) winner = 'O';
    else if (turnState?.status === STATUS_DRAW) winner = 'Draw';

    return {
      totalSteps: this.#frames.length,
      moveCount: turnState?.moveCount ?? 0,
      status: turnState ? statusToLabel(turnState.status) : 'unknown',
      winner,
      startLedger: first?.ledger ?? 0,
      endLedger: last?.ledger ?? 0,
      playerX: players?.playerX,
      playerO: players?.playerO,
    };
  }

  /**
   * Format a single frame as a clean ASCII state log entry.
   */
  formatStateLog(frame = this.current()): string {
    if (!frame) return 'No frame selected.';

    const lines: string[] = [
      `=== Step ${frame.step + 1}/${this.#frames.length} | Ledger ${frame.ledger} [${frame.ledgerClosedAt}] ===`,
      `Action: ${frame.action}`,
      `Event: [${frame.update.family}] component=${frame.update.componentType} entity=${frame.update.entityId} token=${frame.pagingToken}`,
    ];

    if (frame.state.players) {
      lines.push(
        `Players: X = ${frame.state.players.playerX ?? 'unknown'} vs O = ${frame.state.players.playerO ?? 'unknown'}`,
      );
    }

    if (frame.state.turnState) {
      const turn = frame.state.turnState;
      const turnOwner = turn.isXTurn ? 'X' : 'O';
      lines.push(
        `Turn State: moves=${turn.moveCount}, turn=${turnOwner}, status=${turn.statusLabel} (${turn.status})`,
      );
    }

    if (frame.state.board && frame.state.board.length > 0) {
      lines.push('Board:');
      lines.push(formatBoardAscii(frame.state.board));
    }

    return lines.join('\n');
  }

  /**
   * Format the full replay sequence into a readable state timeline log.
   */
  formatFullLog(): string {
    const summary = this.getSummary();
    const header = [
      '=================================================================',
      '               COUGR SPECTATOR REPLAY TIMELINE                   ',
      '=================================================================',
      `Total steps: ${summary.totalSteps} | Moves: ${summary.moveCount} | Status: ${summary.status}`,
      summary.winner ? `Outcome: ${summary.winner === 'Draw' ? 'Draw' : `${summary.winner} Wins!`}` : 'In progress',
      `Ledger span: ${summary.startLedger} -> ${summary.endLedger}`,
      '=================================================================',
      '',
    ].join('\n');

    const framesText = this.#frames.map((frame) => this.formatStateLog(frame)).join('\n\n');
    return `${header}${framesText}\n`;
  }
}
