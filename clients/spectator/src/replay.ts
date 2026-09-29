import {
  decodeCougrEvents,
  decodeCougrPage,
  isRichComponentChanged,
  resolveRichComponentUpdate,
  type CougrEventUpdate,
  type RichComponentChangedUpdate,
  type RichComponentRef,
  type SorobanRpcContractEvent,
  type SorobanRpcEventsPage,
} from 'cougr-sdk-events';

import {
  BOARD_COMPONENT,
  PLAYERS_COMPONENT,
  TURN_STATE_COMPONENT,
  decodeTurnState,
  describeStateTransition,
  type BoardState,
  type PlayersState,
  type TurnState,
} from './turn_based.ts';
import {
  ReplayTimeline,
  type MatchState,
  type TimelineFrame,
} from './timeline.ts';

export interface MatchFixtureFile {
  metadata?: {
    contractId?: string;
    gameType?: string;
    entityId?: number;
    description?: string;
  };
  richStorage?: Record<string, unknown>;
  events: SorobanRpcContractEvent[];
}

export type ReplayRichReader<T = unknown> = (
  ref: RichComponentRef,
  update: RichComponentChangedUpdate,
) => Promise<T>;

export interface ReplayOptions {
  richReader?: ReplayRichReader<unknown>;
  targetEntityId?: number;
}

/**
 * Build a default `ReplayRichReader` from a dictionary of stored states.
 * Key format: `${componentType}:${entityId}:${ledger}` or `${componentType}:${entityId}`.
 */
export function createFixtureRichReader(
  storage: Record<string, unknown> = {},
): ReplayRichReader<unknown> {
  return async ({ componentType, entityId }, update) => {
    const keyWithLedger = `${componentType}:${entityId}:${update.ledger}`;
    if (keyWithLedger in storage) {
      return storage[keyWithLedger];
    }
    const genericKey = `${componentType}:${entityId}`;
    if (genericKey in storage) {
      return storage[genericKey];
    }
    return null;
  };
}

/**
 * Process a sequence of decoded Cougr updates and construct an ordered `ReplayTimeline`.
 *
 * Directly consumes sdk-events types and uses `resolveRichComponentUpdate`
 * for rich event handling.
 */
export async function buildMatchTimeline(
  updates: readonly CougrEventUpdate[],
  options: ReplayOptions = {},
): Promise<ReplayTimeline> {
  const targetEntity = options.targetEntityId ?? 1;
  const richReader = options.richReader;

  // Working state
  let currentTurnState: TurnState | undefined;
  let currentBoard: number[] | undefined;
  let currentPlayers: PlayersState | undefined;
  const genericComponents: Record<string, unknown> = {};

  const frames: TimelineFrame[] = [];

  for (let i = 0; i < updates.length; i += 1) {
    const update = updates[i] as CougrEventUpdate;
    if (update.entityId !== targetEntity) {
      // Still record updates for other entities if any, but focus match state on target
    }

    const prevTurnState = currentTurnState ? { ...currentTurnState } : undefined;
    const prevBoard = currentBoard ? [...currentBoard] : undefined;
    let action = '';

    switch (update.family) {
      case 'set': {
        if (update.componentType === TURN_STATE_COMPONENT) {
          currentTurnState = decodeTurnState(update.data);
          genericComponents[TURN_STATE_COMPONENT] = currentTurnState;
        } else if (update.componentType === BOARD_COMPONENT) {
          // If board is emitted as raw byte markers
          currentBoard = Array.from(update.data);
          genericComponents[BOARD_COMPONENT] = currentBoard;
        } else {
          genericComponents[update.componentType] = update.data;
        }
        break;
      }

      case 'rich': {
        // Must use sdk-events resolveRichComponentUpdate to read out-of-band instance storage
        if (isRichComponentChanged(update)) {
          if (richReader) {
            const resolved = await resolveRichComponentUpdate(update, (ref) =>
              richReader(ref, update),
            );
            const value = resolved.value as Record<string, unknown> | null;
            if (value) {
              if (update.componentType === PLAYERS_COMPONENT) {
                currentPlayers = {
                  playerX: typeof value['playerX'] === 'string' ? value['playerX'] : undefined,
                  playerO: typeof value['playerO'] === 'string' ? value['playerO'] : undefined,
                };
                genericComponents[PLAYERS_COMPONENT] = currentPlayers;
              } else if (update.componentType === BOARD_COMPONENT) {
                if (Array.isArray(value['cells'])) {
                  currentBoard = [...value['cells']];
                  genericComponents[BOARD_COMPONENT] = currentBoard;
                }
              } else {
                genericComponents[update.componentType] = value;
              }
            }
          }
        }
        break;
      }

      case 'del': {
        delete genericComponents[update.componentType];
        if (update.componentType === TURN_STATE_COMPONENT) {
          currentTurnState = undefined;
          action = 'Turn state removed';
        } else if (update.componentType === BOARD_COMPONENT) {
          currentBoard = undefined;
          action = 'Board removed';
        } else if (update.componentType === PLAYERS_COMPONENT) {
          currentPlayers = undefined;
          action = 'Players removed';
        } else {
          action = `Component '${update.componentType}' removed`;
        }
        break;
      }
    }

    if (!action) {
      action = describeStateTransition(
        prevTurnState,
        currentTurnState,
        prevBoard,
        currentBoard,
      );
    }

    // Build immutable snapshot for this step
    const snapshot: MatchState = {
      entityId: targetEntity,
      turnState: currentTurnState ? { ...currentTurnState } : undefined,
      board: currentBoard ? [...currentBoard] : undefined,
      players: currentPlayers ? { ...currentPlayers } : undefined,
      components: { ...genericComponents },
    };

    frames.push({
      step: frames.length,
      ledger: update.ledger,
      ledgerClosedAt: update.ledgerClosedAt,
      txHash: update.txHash,
      pagingToken: update.pagingToken,
      update,
      action,
      state: snapshot,
    });
  }

  return new ReplayTimeline(frames);
}

/**
 * Replay an ordered timeline directly from raw Soroban RPC events using `cougr-sdk-events`.
 */
export async function replayFromEvents(
  events: SorobanRpcContractEvent[],
  options: ReplayOptions = {},
): Promise<ReplayTimeline> {
  // Uses sdk-events decodeCougrEvents directly
  const updates = decodeCougrEvents(events);

  // Stable ordering by ledger then pagingToken if events were collected unordered
  const sorted = [...updates].sort((a, b) => {
    if (a.ledger !== b.ledger) return a.ledger - b.ledger;
    return a.pagingToken.localeCompare(b.pagingToken);
  });

  return buildMatchTimeline(sorted, options);
}

/**
 * Replay an ordered timeline from a full Soroban RPC events page.
 */
export async function replayFromPage(
  page: SorobanRpcEventsPage,
  options: ReplayOptions = {},
): Promise<ReplayTimeline> {
  const { updates } = decodeCougrPage(page);
  return buildMatchTimeline(updates, options);
}

/**
 * Replay an entire match timeline from a committed fixture file.
 */
export async function replayFromFixture(
  fixture: MatchFixtureFile,
  options: ReplayOptions = {},
): Promise<ReplayTimeline> {
  const reader = options.richReader ?? createFixtureRichReader(fixture.richStorage);
  return replayFromEvents(fixture.events, {
    ...options,
    richReader: reader,
  });
}
