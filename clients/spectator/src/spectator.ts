import {
  advanceCursor,
  cougrEventFilter,
  newEventCursor,
  pageCougrEvents,
  parseCursor,
  serializeCursor,
  type CougrEventPage,
  type EventCursor,
  type GetEventsFn,
} from 'cougr-sdk-events';

import {
  buildMatchTimeline,
  type ReplayOptions,
  type ReplayRichReader,
} from './replay.ts';
import { ReplayTimeline } from './timeline.ts';

export interface SpectatorClientOptions {
  contractId?: string;
  startLedger?: number;
  cursor?: EventCursor | string;
  limit?: number;
  richReader?: ReplayRichReader<unknown>;
}

/**
 * Spectator client that tracks an active or completed match using `pageCougrEvents`.
 *
 * Exercises the pagination, topic filtering, cursor tracking, and decoding
 * features of `cougr-sdk-events`.
 */
export class SpectatorClient {
  readonly #contractId?: string;
  readonly #richReader?: ReplayRichReader<unknown>;
  #cursor: EventCursor;
  #timeline: ReplayTimeline = new ReplayTimeline([]);

  constructor(options: SpectatorClientOptions = {}) {
    this.#contractId = options.contractId;
    this.#richReader = options.richReader;

    if (typeof options.cursor === 'string') {
      this.#cursor = parseCursor(options.cursor);
    } else if (options.cursor) {
      this.#cursor = options.cursor;
    } else {
      this.#cursor = newEventCursor({ ledger: options.startLedger ?? null });
    }
  }

  get cursor(): EventCursor {
    return this.#cursor;
  }

  get serializedCursor(): string {
    return serializeCursor(this.#cursor);
  }

  get timeline(): ReplayTimeline {
    return this.#timeline;
  }

  /**
   * Fetch and process new pages from the RPC event stream.
   *
   * Yields each processed `CougrEventPage` as it arrives so consumers can
   * persist cursors or observe frame increments.
   */
  async *spectate(
    getEvents: GetEventsFn,
    options: { maxPages?: number; limit?: number } = {},
  ): AsyncGenerator<CougrEventPage, void, void> {
    const filters = this.#contractId
      ? [cougrEventFilter({ contractIds: [this.#contractId] })]
      : [cougrEventFilter()];

    const iterator = pageCougrEvents(getEvents, {
      filters,
      cursor: this.#cursor,
      limit: options.limit,
      maxPages: options.maxPages,
    });

    for await (const page of iterator) {
      this.#cursor = page.cursor;

      if (page.updates.length > 0) {
        // Append newly discovered updates to current timeline
        const allUpdates = [
          ...this.#timeline.frames.map((f) => f.update),
          ...page.updates,
        ];
        this.#timeline = await buildMatchTimeline(allUpdates, {
          richReader: this.#richReader,
        });
      }

      yield page;
    }
  }

  /**
   * Convenience helper to run spectate until no new events or maxPages is reached.
   */
  async runUntilCaughtUp(
    getEvents: GetEventsFn,
    options: { maxPages?: number; limit?: number } = {},
  ): Promise<ReplayTimeline> {
    for await (const _ of this.spectate(getEvents, options)) {
      // Consume all pages
    }
    return this.#timeline;
  }
}
