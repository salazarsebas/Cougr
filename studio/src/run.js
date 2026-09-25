/**
 * The run: fund the sandbox account, submit a move, wait for Soroban finality,
 * then read the confirmed state — or fail loudly.
 *
 * Three properties are deliberate:
 *
 *   1. The wait is a phase, not an absence of UI. `finalizing` carries the
 *      submitted transaction hash, the elapsed time, the poll count and a
 *      written explanation, so a host can render something that says why the
 *      screen is not moving.
 *   2. `runMatch` never rejects. Every failure becomes an `error` phase with a
 *      descriptor, so a caller cannot accidentally leave a spinner up because
 *      it forgot a `catch`.
 *   3. The poll loop is bounded by `maxPolls`, so a node that accepts a
 *      transaction and never confirms produces a timeout error instead of a
 *      board that spins forever.
 *
 * Failure taxonomy, matching the issue's two named failures plus the timeout
 * the "not a hang" requirement implies:
 *
 *   rpc       — a `get_state` read or a `submitMove` call failed
 *   friendbot — the sandbox account could not be funded on testnet
 *   timeout   — submission succeeded but no confirmed state arrived in budget
 */

import { PHASES } from './state.js';
import { ReadPathTimeoutError, pollForSettlement } from './readPath.js';

/** Default run policy. Override per call, never per module. */
export const DEFAULT_POLICY = Object.freeze({
  pollIntervalMs: 1500,
  finalityTimeoutMs: 30000,
});

const seconds = (ms) => `${(ms / 1000).toFixed(ms % 1000 === 0 ? 0 : 1)}s`;

/**
 * The user-facing descriptor for a failed run. Every field is written to be
 * rendered as-is: the title is a heading, the message explains what happened
 * and what did not, and `recovery` says what the reader can do about it.
 */
export function describeRunError(kind, context = {}) {
  const timeoutMs = context.timeoutMs ?? DEFAULT_POLICY.finalityTimeoutMs;
  const detail = context.detail ? String(context.detail) : null;

  switch (kind) {
    case 'friendbot':
      return {
        kind,
        title: 'Sandbox account could not be funded',
        message:
          'Friendbot did not fund the studio\'s testnet account, so there is nothing to play against. '
          + 'No move was submitted and nothing on chain changed.',
        recovery: 'Try again in a moment — friendbot is rate limited, and a second attempt usually succeeds.',
        retryable: true,
        detail,
      };

    case 'rpc':
      return {
        kind,
        title: 'Soroban RPC request failed',
        message:
          'The studio could not talk to the Soroban RPC endpoint, so it has no confirmed state to draw. '
          + 'The run stopped on the failure instead of leaving the board spinning.',
        recovery: 'Check the RPC endpoint and the network, then run the move again.',
        retryable: true,
        detail,
      };

    case 'timeout':
      return {
        kind,
        title: 'Timed out waiting for finality',
        message:
          `No confirmed state arrived within ${seconds(timeoutMs)}, so the studio stopped waiting. `
          + 'The transaction may still land — re-check the board before retrying, so a late move is not sent twice.',
        recovery: 'Run Move again to re-read the state, or wait a moment and retry the move.',
        retryable: true,
        detail,
      };

    default:
      return {
        kind: kind ?? 'unknown',
        title: 'The run failed',
        message: 'The studio hit an unexpected failure and stopped the run.',
        recovery: 'Run the move again.',
        retryable: true,
        detail,
      };
  }
}

/** The model every consumer (view, fixtures, host) reads. */
export function createRunModel(partial = {}) {
  return {
    phase: PHASES.IDLE,
    config: partial.config ?? null,
    state: partial.state ?? null,
    previousState: partial.previousState ?? null,
    move: partial.move ?? null,
    txHash: partial.txHash ?? null,
    error: partial.error ?? null,
    elapsedMs: partial.elapsedMs ?? 0,
    polls: partial.polls ?? 0,
    maxPolls: partial.maxPolls ?? null,
    ...partial,
  };
}

/**
 * Run one move against a client and resolve with the final model.
 *
 * The client is injected, which is what makes the whole flow testable without
 * a chain:
 *
 *   {
 *     ensureFunded(): Promise<void>            // friendbot — may reject
 *     submitMove(move): Promise<{ txHash }>    // contract call — may reject
 *     readState(): Promise<MatchState>         // get_state — may reject
 *   }
 *
 * `ensureFunded` is optional: a host that funds once per session passes a
 * client without it and the funding phase is skipped.
 *
 * @param {object} args
 * @param {object} args.config
 * @param {object} args.client
 * @param {object} [args.move]
 * @param {object|null} [args.previousState] the last confirmed state, the baseline for change detection
 * @param {object} [args.policy]
 * @param {object} [args.hooks] `onPhase(model)` is called for every transition, including the first
 * @returns {Promise<object>} the settled or errored model; never rejects
 */
export async function runMatch({
  config,
  client,
  move = null,
  previousState = null,
  policy = {},
  hooks = {},
} = {}) {
  if (!client || typeof client.submitMove !== 'function' || typeof client.readState !== 'function') {
    throw new TypeError('runMatch requires a client with submitMove and readState');
  }

  const settings = { ...DEFAULT_POLICY, ...policy };
  const wait = settings.wait ?? ((ms) => new Promise((resolve) => setTimeout(resolve, ms)));
  const now = settings.now ?? (() => Date.now());
  const maxPolls = settings.maxPolls
    ?? Math.max(1, Math.ceil(settings.finalityTimeoutMs / settings.pollIntervalMs));
  const startedAt = now();

  const emit = (model) => {
    if (typeof hooks.onPhase === 'function') hooks.onPhase(model);
    return model;
  };

  let model = createRunModel({ config, state: previousState, previousState, move, maxPolls });

  const failed = (phaseModel, kind, cause) => emit(createRunModel({
    ...phaseModel,
    phase: PHASES.ERROR,
    elapsedMs: now() - startedAt,
    error: describeRunError(kind, {
      timeoutMs: settings.finalityTimeoutMs,
      detail: cause instanceof Error ? cause.message : cause,
    }),
  }));

  if (typeof client.ensureFunded === 'function') {
    emit(createRunModel({ ...model, phase: PHASES.FUNDING }));
    try {
      await client.ensureFunded();
    } catch (cause) {
      return failed(model, 'friendbot', cause);
    }
  }

  model = emit(createRunModel({ ...model, phase: PHASES.SUBMITTING }));
  let txHash = null;
  try {
    const result = await client.submitMove(move);
    txHash = result && result.txHash ? result.txHash : null;
  } catch (cause) {
    return failed(model, 'rpc', cause);
  }

  model = emit(createRunModel({ ...model, phase: PHASES.FINALIZING, txHash }));
  try {
    const settled = await pollForSettlement({
      readState: client.readState,
      previousState,
      pollIntervalMs: settings.pollIntervalMs,
      maxPolls,
      finalityTimeoutMs: settings.finalityTimeoutMs,
      wait,
      now,
      onPoll: (nextState, polls) => {
        emit(createRunModel({
          ...model,
          phase: PHASES.FINALIZING,
          txHash,
          state: nextState,
          polls,
          elapsedMs: now() - startedAt,
        }));
      },
    });

    return emit(createRunModel({
      ...model,
      phase: PHASES.SETTLED,
      txHash,
      state: settled.state,
      polls: settled.polls,
      elapsedMs: now() - startedAt,
    }));
  } catch (cause) {
    if (cause instanceof ReadPathTimeoutError) return failed(model, 'timeout', cause);
    return failed(model, 'rpc', cause);
  }
}
