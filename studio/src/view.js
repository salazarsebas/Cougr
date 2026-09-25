/**
 * Studio view — assembles the board, the match status, the run panel and the
 * sandbox warning into one HTML fragment.
 *
 * Everything is a string. The view has no DOM dependency, so the same code
 * runs in `node:test` (where the assertions live), in the fixture renderer
 * that writes artifacts, and in a host page via `innerHTML`. A host that wants
 * a framework component wraps this; it does not reimplement it.
 *
 * The four things the issue asks the screen to say are all here:
 *   - the board, at the config's size
 *   - a deliberate wait during the finality window, with the reason written out
 *   - a real error for RPC and friendbot failure, never a hang
 *   - a visible testnet-sandbox warning
 */

import { renderBoard } from './board.js';
import { cellCount, markLabel, playerToMove } from './config.js';
import { PHASES, TERMINAL_PHASES, filledCells, isTerminal, statusLabel } from './state.js';
import { BRAND, LINE, STUDIO, TINT, px, withAlpha } from './theme.js';

/**
 * The sandbox warning. It is a constant, not a prop: the studio only ever
 * talks to testnet, so there is no code path where the warning would be wrong,
 * and no caller can forget it.
 */
export const SANDBOX_WARNING =
  'Testnet sandbox — a rehearsal, not real money. This board runs against Stellar testnet. '
  + 'There are no real funds, no real assets and nothing here has value.';

/** Human text per phase, so a host never has to invent copy. */
export const PHASE_COPY = Object.freeze({
  [PHASES.IDLE]: {
    title: 'Ready to run',
    detail: 'Run a move to send it to the testnet sandbox and watch the board settle.',
  },
  [PHASES.FUNDING]: {
    title: 'Funding the sandbox account',
    detail: 'Asking friendbot for testnet XLM so the move can be submitted. Nothing is spent — this is testnet.',
  },
  [PHASES.SUBMITTING]: {
    title: 'Submitting the move',
    detail: 'Signing and sending the transaction to Soroban. The board above is still the last confirmed state.',
  },
  [PHASES.FINALIZING]: {
    title: 'Waiting for Soroban finality',
    detail:
      'The move is submitted. Stellar closes a ledger roughly every 5 seconds, and the studio reads the contract again '
      + 'after each close and repaints when the state changes. This wait is deliberate — the board is not frozen.',
  },
  [PHASES.SETTLED]: {
    title: 'Settled',
    detail: 'The ledger closed and the contract returned the new state. The board above is confirmed on testnet.',
  },
});

/** CSS for the fragment. Scoped under `.cougr-studio` so it cannot leak. */
export function studioStyles() {
  return `
.cougr-studio { box-sizing: border-box; display: flex; flex-direction: column; gap: ${px(BRAND.space4)}px; max-width: ${renderBoardMaxWidth()}px; padding: ${px(BRAND.space4)}px; background: ${BRAND.colorBg}; color: ${BRAND.colorText}; font-family: ${BRAND.fontSans}; border: 1px solid ${withAlpha(LINE.stroke, LINE.borderOpacity)}; border-radius: ${px(BRAND.radiusLg)}px; }
.cougr-studio *, .cougr-studio *::before, .cougr-studio *::after { box-sizing: border-box; }
.cougr-studio__sandbox { margin: 0; padding: ${px(BRAND.space3)}px ${px(BRAND.space4)}px; background: ${STUDIO.waiting}${TINT}; color: ${BRAND.colorText}; border-left: 3px solid ${STUDIO.waiting}; border-radius: ${px(BRAND.radiusSm)}px; font-size: 13px; line-height: 1.5; }
.cougr-studio__header { display: flex; flex-wrap: wrap; align-items: baseline; gap: ${px(BRAND.space2)}px ${px(BRAND.space4)}px; }
.cougr-studio__title { margin: 0; font-size: 18px; font-weight: 700; }
.cougr-studio__caption { margin: 0; color: ${BRAND.colorTextSecondary}; font-size: 13px; }
.cougr-studio__board { display: block; line-height: 0; }
.cougr-studio__board svg { max-width: 100%; height: auto; }
.cougr-studio__status { display: flex; flex-wrap: wrap; align-items: center; gap: ${px(BRAND.space2)}px ${px(BRAND.space4)}px; margin: 0; }
.cougr-studio__status-badge { padding: ${px(BRAND.space1)}px ${px(BRAND.space3)}px; border-radius: ${px(BRAND.radiusFull)}px; font-size: 13px; font-weight: 700; }
.cougr-studio__status-meta { color: ${BRAND.colorTextSecondary}; font-size: 12px; }
.cougr-studio__run { padding: ${px(BRAND.space3)}px ${px(BRAND.space4)}px; background: ${BRAND.colorSurface}; border: 1px solid ${LINE.stroke}; border-radius: ${px(BRAND.radiusMd)}px; font-size: 13px; line-height: 1.55; }
.cougr-studio__run[data-run-phase='finalizing'] { border-color: ${STUDIO.waiting}; }
.cougr-studio__run[data-run-phase='error'] { border-color: ${STUDIO.danger}; }
.cougr-studio__run-title { display: flex; align-items: center; gap: ${px(BRAND.space2)}px; margin: 0 0 ${px(BRAND.space2)}px; font-size: 14px; font-weight: 700; }
.cougr-studio__run-detail { margin: 0; color: ${BRAND.colorTextSecondary}; }
.cougr-studio__run-meta { margin: ${px(BRAND.space2)}px 0 0; color: ${BRAND.colorTextSecondary}; font-size: 12px; font-family: ${BRAND.fontMono}; word-break: break-all; }
.cougr-studio__spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid ${STUDIO.waiting}; border-right-color: transparent; border-radius: ${px(BRAND.radiusFull)}px; animation: cougr-studio-spin 900ms linear infinite; }
@keyframes cougr-studio-spin { to { transform: rotate(360deg); } }
@media (prefers-reduced-motion: reduce) { .cougr-studio__spinner { animation: none; border-right-color: ${STUDIO.waiting}; opacity: 0.5; } }
.cougr-studio__error-title { color: ${STUDIO.danger}; }
.cougr-studio__retry { margin-top: ${px(BRAND.space3)}px; padding: ${px(BRAND.space2)}px ${px(BRAND.space4)}px; background: ${BRAND.colorAccent}; color: ${BRAND.colorBg}; border: 0; border-radius: ${px(BRAND.radiusMd)}px; font: inherit; font-weight: 700; cursor: pointer; }
.cougr-studio__retry:focus-visible { outline: 2px solid ${BRAND.colorAccent}; outline-offset: 2px; }
`.trim();
}

/** Pixel cap for the fragment, derived from the board so the two line up. */
function renderBoardMaxWidth() {
  return 480 + px(BRAND.space4) * 2 + 2;
}

/**
 * Render the whole studio view.
 *
 * @param {object} model a model from `runMatch` or a fixture
 * @param {object} [options]
 * @param {boolean} [options.inlineStyles=true] include the scoped stylesheet
 * @param {object}  [options.boardOptions] forwarded to `renderBoard`
 */
export function renderStudioView(model, options = {}) {
  const { config, state, phase } = model;

  if (!config) throw new TypeError('renderStudioView requires a model with a config');

  const parts = [];
  parts.push(`<section class="cougr-studio" data-phase="${escapeAttribute(phase)}" data-testid="cougr-studio">`);

  if (options.inlineStyles !== false) parts.push(`<style>${studioStyles()}</style>`);

  parts.push(`<p class="cougr-studio__sandbox" role="note" data-testid="sandbox-warning">${escapeText(SANDBOX_WARNING)}</p>`);

  parts.push('<header class="cougr-studio__header">');
  parts.push(`<h2 class="cougr-studio__title">${escapeText(config.label)}</h2>`);
  parts.push(`<p class="cougr-studio__caption" data-testid="board-caption">${escapeText(boardCaption(model))}</p>`);
  parts.push('</header>');

  parts.push(`<div class="cougr-studio__board" data-testid="board">${renderBoard(config, state, options.boardOptions)}</div>`);

  parts.push(renderStatusLine(model));
  parts.push(renderRunPanel(model));

  parts.push('</section>');
  return parts.join('');
}

/** The one-line status under the board: who is winning, who moves, cells used. */
export function renderStatusLine(model) {
  const { config, state } = model;
  const label = state ? statusLabel(config, state) : 'Waiting for the first read';
  const color = state && isTerminal(config, state) ? STUDIO.settled : BRAND.colorPrimary;
  const meta = state
    ? `${filledCells(config, state)}/${cellCount(config)} cells · ${state.is_x_turn === false ? 'second player' : 'first player'} to move`
    : `0/${cellCount(config)} cells`;

  return [
    '<p class="cougr-studio__status" data-testid="status-line">',
    `<span class="cougr-studio__status-badge" style="background:${color}${TINT};color:${color}">${escapeText(label)}</span>`,
    `<span class="cougr-studio__status-meta" data-testid="status-meta">${escapeText(meta)}</span>`,
    '</p>',
  ].join('');
}

/**
 * The run panel: what the studio is doing, why, and what to do if it failed.
 * This is the element that has to make the finality wait read as deliberate and
 * an RPC or friendbot failure read as an error.
 */
export function renderRunPanel(model) {
  const { phase } = model;

  if (phase === PHASES.ERROR) return renderErrorPanel(model);
  if (phase === PHASES.FINALIZING) return renderFinalizingPanel(model);

  const copy = PHASE_COPY[phase] ?? PHASE_COPY[PHASES.IDLE];
  const busy = phase === PHASES.FUNDING || phase === PHASES.SUBMITTING;
  const attributes = busy
    ? 'role="status" aria-live="polite"'
    : 'role="status"';

  return [
    `<div class="cougr-studio__run" data-run-phase="${escapeAttribute(phase)}" ${attributes}>`,
    '<h3 class="cougr-studio__run-title">',
    busy ? '<span class="cougr-studio__spinner" aria-hidden="true"></span>' : '',
    escapeText(copy.title),
    '</h3>',
    `<p class="cougr-studio__run-detail">${escapeText(copy.detail)}</p>`,
    renderRunMeta(model),
    '</div>',
  ].join('');
}

/** The deliberate wait: spinner, written reason, and the numbers behind it. */
function renderFinalizingPanel(model) {
  const copy = PHASE_COPY[PHASES.FINALIZING];
  const elapsed = `${(model.elapsedMs / 1000).toFixed(1)}s`;
  const budget = model.maxPolls ? `${model.maxPolls} reads max` : 'bounded';

  return [
    '<div class="cougr-studio__run" data-run-phase="finalizing" data-testid="finality-wait" role="status" aria-live="polite">',
    `<h3 class="cougr-studio__run-title"><span class="cougr-studio__spinner" aria-hidden="true"></span>${escapeText(copy.title)}</h3>`,
    `<p class="cougr-studio__run-detail" data-testid="finality-explanation">${escapeText(copy.detail)}</p>`,
    `<p class="cougr-studio__run-meta" data-testid="finality-meta">waiting ${escapeText(elapsed)} · ${escapeText(String(model.polls))} reads · ${escapeText(budget)}${model.txHash ? ` · tx ${escapeText(model.txHash)}` : ''}</p>`,
    '</div>',
  ].join('');
}

/** An error, stated plainly, with a retry — and an explicit "not a hang" line. */
function renderErrorPanel(model) {
  const error = model.error ?? { title: 'The run failed', message: '', recovery: '', detail: null };

  return [
    `<div class="cougr-studio__run" data-run-phase="error" data-error-kind="${escapeAttribute(error.kind ?? 'unknown')}" data-testid="run-error" role="alert">`,
    `<h3 class="cougr-studio__run-title cougr-studio__error-title">${escapeText(error.title)}</h3>`,
    `<p class="cougr-studio__run-detail" data-testid="error-message">${escapeText(error.message)}</p>`,
    error.recovery ? `<p class="cougr-studio__run-detail">${escapeText(error.recovery)}</p>` : '',
    '<p class="cougr-studio__run-detail">The run stopped and released the wait — it does not retry in the background.</p>',
    error.detail ? `<p class="cougr-studio__run-meta" data-testid="error-detail">${escapeText(error.detail)}</p>` : '',
    '<button type="button" class="cougr-studio__retry" data-action="retry">Retry move</button>',
    '</div>',
  ].join('');
}

/** Elapsed/polls/tx line for phases that settled or are still working. */
function renderRunMeta(model) {
  if (TERMINAL_PHASES.includes(model.phase) && model.phase !== PHASES.ERROR) {
    const elapsed = `${(model.elapsedMs / 1000).toFixed(1)}s`;
    const tx = model.txHash ? ` · tx ${model.txHash}` : '';
    return `<p class="cougr-studio__run-meta" data-testid="run-meta">settled in ${escapeText(elapsed)} after ${escapeText(String(model.polls))} reads${escapeText(tx)}</p>`;
  }

  if (model.txHash) {
    return `<p class="cougr-studio__run-meta" data-testid="run-meta">tx ${escapeText(model.txHash)}</p>`;
  }

  return '';
}

function boardCaption(model) {
  const { config, state } = model;
  if (!state) return `${config.board.width}×${config.board.height} · no confirmed state yet`;
  const mover = markLabel(config, playerToMove(config, state));
  return `${config.board.width}×${config.board.height} · move ${state.move_count} · ${mover} to move`;
}

function escapeText(value) {
  return String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function escapeAttribute(value) {
  return escapeText(value).replace(/"/g, '&quot;');
}
