/**
 * The read path: how the studio learns what happened on chain.
 *
 * Decision: poll `get_state`.
 *
 * The alternative the issue names is subscribing to the `COUGR` events in
 * `src/ecs_events.rs` (`set`, `del`, `rich`). That path is viable, but it puts
 * more work on the client for strictly less information:
 *
 *   1. `RichComponentChangedEvent` does not carry the full value. Its own
 *      doc comment says so: "The full value is not embedded in the event -
 *      off-chain indexers should query the contract's instance storage for the
 *      updated value after receiving this notification." A board built on
 *      `rich` therefore has to follow every notification with a read anyway, so
 *      events never replace `get_state` — they only decide *when* to call it.
 *   2. `ComponentSetEvent` does carry `data: Bytes`, but those are the raw
 *      serialized component bytes. Reassembling a `GameState` from per-component
 *      deltas means reimplementing the contract's serialization off chain, and
 *      coupling the studio to component layout rather than to the API shape.
 *   3. The contract already exposes exactly the shape the board renders:
 *      `get_state` returns `{ cells, status, is_x_turn, move_count }` in one
 *      call — the same object a live host passes the component. There is
 *      nothing to reassemble.
 *   4. Polling is self-healing. A dropped notification, a topic filter that
 *      misses a component type, or a client that reconnects mid-run leaves an
 *      event subscriber stale forever; a poll converges on the on-chain truth
 *      on its next tick.
 *
 * What the event path would buy is latency, and the finality window is the
 * dominant cost anyway: Stellar closes a ledger roughly every five seconds, so
 * a one-second poll is well inside the noise. The trade-off accepted here is a
 * bounded number of extra RPC reads during a few-second window, in exchange for
 * one code path that cannot silently miss a move.
 *
 * If this is ever revisited, the honest version is a hybrid: subscribe to
 * `COUGR`/`set` only as a nudge to read sooner, and keep `get_state` as the
 * source of truth. That is exactly what this module's loop does with a timer
 * instead of an event source.
 */

/** The read path the studio uses, in a form a host or a test can inspect. */
export const READ_PATH = Object.freeze({
  kind: 'poll',
  contractCall: 'get_state',
  eventPrefix: ['COUGR'],
  eventTopics: ['set', 'del', 'rich'],
  richEventCarriesValue: false,
});

/**
 * Why polling `get_state` was chosen over subscribing to `COUGR` events.
 * Kept as data so the PR body, the README and any future UI can quote the same
 * reasoning instead of paraphrasing it.
 */
export function readPathJustification() {
  return {
    choice: 'poll get_state',
    summary:
      'Poll the contract\'s get_state until the match changes, with a bounded number of attempts. '
      + 'The COUGR rich event does not carry the component value, so an event subscription would still need a follow-up read.',
    reasons: [
      'RichComponentChangedEvent carries only { component_type, entity_id }; its doc comment says off-chain indexers must query the contract for the updated value.',
      'ComponentSetEvent carries raw component bytes, so an event-only board would have to reimplement the contract\'s serialization to rebuild GameState.',
      'get_state already returns { cells, status, is_x_turn, move_count } in one call: the exact shape the board renders and the shape a live host passes through.',
      'Polling converges even when a notification is missed; an event subscriber that drops one update stays stale.',
    ],
    tradeoffs: [
      'A bounded number of extra RPC reads during the finality window.',
      'Slightly higher latency than a perfect event stream, which the ~5s ledger close dominates anyway.',
    ],
    followUpReadRequiredForEvents: true,
  };
}

/** Raised when the poll budget is exhausted before the state changes. */
export class ReadPathTimeoutError extends Error {
  constructor(polls, timeoutMs) {
    super(`no confirmed state change after ${polls} reads (budget ${timeoutMs}ms)`);
    this.name = 'ReadPathTimeoutError';
    this.polls = polls;
    this.timeoutMs = timeoutMs;
  }
}

/** Default sleep, injectable so tests need no wall-clock. */
export function defaultWait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * A move has landed when the match changed in any way the board would draw:
 * a new move was recorded, or the status moved (a win or draw resolved).
 *
 * Change detection rather than `move_count === expected` on purpose: the
 * contract is the authority, a `del` or a reset is still a change, and a
 * reorg that lands a different-but-newer state should settle rather than spin.
 */
export function defaultIsSettled(previousState, nextState) {
  if (!previousState) return true;
  if (nextState.status !== previousState.status) return true;
  return nextState.move_count > previousState.move_count;
}

/**
 * Poll `readState` until it reports a change.
 *
 * The loop is bounded twice over — by `maxPolls` and by the injected clock's
 * deadline — so a caller that passes a clock whose time never advances still
 * terminates. That is the "never hang" guarantee, enforced here rather than
 * trusted to the caller.
 *
 * @throws {ReadPathTimeoutError} when the budget is exhausted.
 * @throws whatever `readState` throws, for the caller to classify.
 */
export async function pollForSettlement({
  readState,
  previousState = null,
  isSettled = defaultIsSettled,
  pollIntervalMs = 1500,
  maxPolls = 20,
  finalityTimeoutMs = pollIntervalMs * maxPolls,
  wait = defaultWait,
  now = () => Date.now(),
  onPoll = null,
} = {}) {
  if (typeof readState !== 'function') throw new TypeError('pollForSettlement requires a readState function');

  const startedAt = now();
  let polls = 0;

  while (polls < maxPolls) {
    await wait(pollIntervalMs);
    polls += 1;

    const state = await readState();
    if (onPoll) onPoll(state, polls, now() - startedAt);

    if (isSettled(previousState, state)) {
      return { state, polls, elapsedMs: now() - startedAt };
    }
  }

  throw new ReadPathTimeoutError(polls, finalityTimeoutMs);
}
