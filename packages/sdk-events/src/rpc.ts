import type { SorobanRpcEventsPage } from './events.ts';
import type { SorobanEventFilter } from './topics.ts';

/** Minimal structural `fetch` contract, so the SDK works in any runtime. */
export interface FetchLike {
  (input: string, init: { method: string; headers: Record<string, string>; body: string }): Promise<{
    ok: boolean;
    status: number;
    json(): Promise<unknown>;
    text(): Promise<string>;
  }>;
}

export interface SorobanRpcClientOptions {
  /** Soroban RPC endpoint, e.g. `https://soroban-testnet.stellar.org`. */
  url: string;
  /** Defaults to the global `fetch`. Inject this for tests or custom signing. */
  fetch?: FetchLike;
  headers?: Record<string, string>;
}

export interface GetEventsRequest {
  /** Ledger to start from when there is no resume cursor. */
  startLedger?: number;
  endLedger?: number;
  /** `pagingToken` from a previous response; takes precedence over `startLedger`. */
  cursor?: string;
  filters?: SorobanEventFilter[];
  /** 1..10000, RPC default 100. */
  limit?: number;
}

export type GetEventsResponse = SorobanRpcEventsPage;

export type GetEventsFn = (request: GetEventsRequest) => Promise<GetEventsResponse>;

interface JsonRpcResponse {
  result?: unknown;
  error?: { code?: number; message?: string; data?: unknown };
}

/**
 * Create a `getEvents` function bound to a Soroban RPC endpoint.
 *
 * ```ts
 * const getEvents = createGetEvents({ url: 'https://soroban-testnet.stellar.org' });
 * const page = await getEvents({ startLedger: 1000, filters: [cougrEventFilter()] });
 * ```
 */
export function createGetEvents(options: SorobanRpcClientOptions): GetEventsFn {
  const doFetch = options.fetch ?? (globalThis.fetch as unknown as FetchLike | undefined);
  if (doFetch === undefined) {
    throw new Error('No fetch implementation available; pass options.fetch');
  }

  return async (request: GetEventsRequest): Promise<GetEventsResponse> => {
    const params: Record<string, unknown> = {};
    if (request.cursor !== undefined) params['cursor'] = request.cursor;
    if (request.startLedger !== undefined) params['startLedger'] = request.startLedger;
    if (request.endLedger !== undefined) params['endLedger'] = request.endLedger;
    if (request.filters !== undefined) params['filters'] = request.filters;
    if (request.limit !== undefined) params['limit'] = request.limit;

    const payload = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'getEvents',
      params: [params],
    });

    const response = await doFetch(options.url, {
      method: 'POST',
      headers: { 'content-type': 'application/json', ...(options.headers ?? {}) },
      body: payload,
    });

    if (!response.ok) {
      throw new Error(`getEvents HTTP ${response.status}: ${await response.text()}`);
    }

    const body = (await response.json()) as JsonRpcResponse;
    if (body.error !== undefined) {
      throw new Error(`getEvents RPC error ${body.error.code ?? ''}: ${body.error.message ?? 'unknown'}`);
    }
    const result = body.result;
    if (typeof result !== 'object' || result === null || !Array.isArray((result as SorobanRpcEventsPage).events)) {
      throw new Error('getEvents returned an unexpected result shape');
    }
    return result as SorobanRpcEventsPage;
  };
}
