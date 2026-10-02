import {
  decodeCougrEvents,
  type CougrEventUpdate,
  type RichComponentChangedUpdate,
  type SorobanRpcEventsPage,
} from './events.ts';
import type { GetEventsFn } from './rpc.ts';
import type { SorobanEventFilter } from './topics.ts';

/**
 * A resumable position in the Soroban event stream.
 *
 * Persist this after every processed page (it is `JSON.stringify`-safe) and
 * hand it back on startup. `pagingToken` is what Soroban RPC expects as the
 * `cursor` request parameter; `ledger` is kept so a client that lost the token
 * can still fall back to `startLedger`.
 */
export interface EventCursor {
  pagingToken: string | null;
  ledger: number | null;
}

export function newEventCursor(init: Partial<EventCursor> = {}): EventCursor {
  return {
    pagingToken: init.pagingToken ?? null,
    ledger: init.ledger ?? null,
  };
}

export function serializeCursor(cursor: EventCursor): string {
  return JSON.stringify(cursor);
}

export function parseCursor(serialized: string): EventCursor {
  const parsed: unknown = JSON.parse(serialized);
  if (typeof parsed !== 'object' || parsed === null) {
    throw new TypeError('Cursor JSON must be an object');
  }
  const record = parsed as { pagingToken?: unknown; ledger?: unknown };
  return newEventCursor({
    pagingToken: typeof record.pagingToken === 'string' ? record.pagingToken : null,
    ledger: typeof record.ledger === 'number' ? record.ledger : null,
  });
}

/**
 * Advance a cursor over one RPC page.
 *
 * Prefers the page-level `cursor` the node returns; falls back to the last
 * event's `pagingToken`; finally keeps the previous token so a non-advancing
 * page cannot silently rewind the stream.
 */
export function advanceCursor(
  cursor: EventCursor,
  page: SorobanRpcEventsPage,
  updates: readonly CougrEventUpdate[] = decodeCougrEvents(page.events),
): EventCursor {
  const lastUpdate = updates.length > 0 ? updates[updates.length - 1] : undefined;
  const pagingToken = page.cursor ?? lastUpdate?.pagingToken ?? cursor.pagingToken;
  const ledger = lastUpdate?.ledger ?? cursor.ledger;
  return newEventCursor({ pagingToken, ledger });
}

export interface CougrPageOptions {
  /** Filter passed through to `getEvents`; defaults to all three families. */
  filters?: SorobanEventFilter[];
  /** Required on a cold start; ignored when `cursor.pagingToken` is set. */
  startLedger?: number;
  limit?: number;
  cursor?: EventCursor;
  /** Safety valve for tests and bounded backfills. */
  maxPages?: number;
  /** Paging tokens already handled by the client; skipped on resume. */
  seenPagingTokens?: Set<string>;
}

export interface CougrEventPage {
  /** New, deduplicated Cougr updates from this page. */
  updates: CougrEventUpdate[];
  /** Persist this and pass it back to resume exactly after this page. */
  cursor: EventCursor;
  latestLedger: number;
  /** The undecoded RPC page, for callers that want other event families. */
  page: SorobanRpcEventsPage;
}

/**
 * Walk `getEvents` page by page, decoding each page into typed Cougr updates.
 *
 * The generator yields after every page so a consumer can persist
 * `page.cursor` before asking for more work. Breaking out of the loop is safe:
 * resuming with the yielded cursor continues from the next page.
 */
export async function* pageCougrEvents(
  getEvents: GetEventsFn,
  options: CougrPageOptions = {},
): AsyncGenerator<CougrEventPage, void, void> {
  let cursor = newEventCursor(options.cursor ?? { ledger: options.startLedger ?? null });
  const seen = options.seenPagingTokens ?? new Set<string>();
  let pages = 0;

  while (true) {
    const request: Parameters<GetEventsFn>[0] = {
      filters: options.filters,
      limit: options.limit,
    };
    if (cursor.pagingToken !== null) {
      request.cursor = cursor.pagingToken;
    } else if (cursor.ledger !== null) {
      request.startLedger = cursor.ledger;
    } else {
      throw new TypeError('pageCougrEvents needs a startLedger or a resume cursor');
    }

    const page = await getEvents(request);
    const decoded = decodeCougrEvents(page.events);
    const fresh: CougrEventUpdate[] = [];
    for (const update of decoded) {
      if (seen.has(update.pagingToken)) continue;
      seen.add(update.pagingToken);
      fresh.push(update);
    }

    const next = advanceCursor(cursor, page, decoded);
    yield { updates: fresh, cursor: next, latestLedger: page.latestLedger, page };

    pages += 1;
    if (options.maxPages !== undefined && pages >= options.maxPages) return;
    if (page.events.length === 0) return;
    if (next.pagingToken === cursor.pagingToken && next.pagingToken !== null) return;
    cursor = next;
  }
}

export interface RichComponentRef {
  contractId?: string;
  componentType: string;
  entityId: number;
}

export type RichComponentReader<T> = (ref: RichComponentRef) => Promise<T>;

/**
 * Complete a rich-component update with the value that was deliberately left
 * out of the event.
 *
 * Rich components live in the contract's instance storage, keyed by
 * `(cougr_rc, entity_id, component_type)`. That key is an implementation
 * detail, so the SDK does not guess it: pass a reader that calls the
 * contract's own read system (or `getLedgerEntries` with the key you know).
 */
export async function resolveRichComponentUpdate<T>(
  update: RichComponentChangedUpdate,
  read: RichComponentReader<T>,
): Promise<{ update: RichComponentChangedUpdate; value: T }> {
  const ref: RichComponentRef = {
    componentType: update.followUp.componentType,
    entityId: update.followUp.entityId,
  };
  if (update.followUp.contractId !== undefined) ref.contractId = update.followUp.contractId;
  const value = await read(ref);
  return { update, value };
}
