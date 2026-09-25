import test from 'node:test';
import assert from 'node:assert/strict';

import { BOARD_MAX_PX } from '../src/board.js';
import { defaultConfig } from '../src/config.js';
import { createRunModel, describeRunError } from '../src/run.js';
import { PHASES } from '../src/state.js';
import { PHASE_COPY, SANDBOX_WARNING, renderStudioView, studioStyles } from '../src/view.js';
import { listFixtures, loadFixture, modelFromFixture } from '../tools/fixtures.js';

const count = (haystack, needle) => haystack.split(needle).length - 1;

test('every fixture renders the sandbox warning, because it is not a prop', () => {
  for (const id of listFixtures()) {
    const html = renderStudioView(modelFromFixture(loadFixture(id)));
    assert.ok(html.includes(SANDBOX_WARNING), `${id} is missing the sandbox warning`);
    assert.match(html, /data-testid="sandbox-warning"/);
    assert.match(html, /role="note"/);
  }

  assert.match(SANDBOX_WARNING, /testnet/i);
  assert.match(SANDBOX_WARNING, /not real money/i);
});

test('the 3x3 win fixture draws nine cells and the winning status', () => {
  const model = modelFromFixture(loadFixture('3x3-win'));
  const html = renderStudioView(model);

  assert.equal(count(html, 'data-cell-index='), 9);
  assert.match(html, /data-cols="3" data-rows="3"/);
  assert.match(html, /data-testid="status-line"/);
  assert.match(html, />X wins</);
  assert.equal(count(html, 'data-winning="true"'), 3);
});

test('the larger board fixture draws all 42 cells with a non-square caption', () => {
  const model = modelFromFixture(loadFixture('7x6-win'));
  const html = renderStudioView(model);

  assert.equal(count(html, 'data-cell-index='), 42);
  assert.match(html, /data-cols="7" data-rows="6"/);
  assert.match(html, /data-testid="board-caption"[^>]*>7×6/);
  assert.match(html, />R wins</);
});

test('the draw fixture says Draw and highlights nothing', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-draw')));

  assert.match(html, />Draw</);
  assert.equal(count(html, 'data-winning="true"'), 0);
  assert.match(html, /data-testid="status-meta"[^>]*>9\/9 cells/);
});

test('the wait state is visible and explains why the board is not moving', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-finalizing')));

  assert.match(html, /data-run-phase="finalizing"/);
  assert.match(html, /data-testid="finality-wait"/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /cougr-studio__spinner/);
  assert.ok(html.includes(PHASE_COPY[PHASES.FINALIZING].title));
  assert.ok(html.includes(PHASE_COPY[PHASES.FINALIZING].detail));
  assert.match(html, /deliberate/);
  assert.match(html, /data-testid="finality-meta"[^>]*>waiting 3\.2s · 2 reads · 20 reads max/);
  assert.match(html, /tx d4e7c1f0/);

  // The board is still drawn during the wait, so the screen is never empty.
  assert.equal(count(html, 'data-cell-index='), 9);
});

test('an RPC failure renders an error, not a wait', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-rpc-error')));

  assert.match(html, /data-run-phase="error"/);
  assert.match(html, /data-error-kind="rpc"/);
  assert.match(html, /data-testid="run-error"/);
  assert.match(html, /role="alert"/);
  assert.match(html, /Soroban RPC request failed/);
  assert.match(html, /does not retry in the background/);
  assert.match(html, /data-action="retry"/);
  assert.match(html, /ECONNRESET/);

  assert.equal(html.includes('data-testid="finality-wait"'), false);
  assert.equal(html.includes(PHASE_COPY[PHASES.FINALIZING].title), false);
});

test('a friendbot failure renders an error that says nothing was submitted', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-friendbot-error')));

  assert.match(html, /data-error-kind="friendbot"/);
  assert.match(html, /Sandbox account could not be funded/);
  assert.match(html, /No move was submitted/);
  assert.match(html, /429 Too Many Requests/);
  assert.equal(html.includes('data-testid="finality-wait"'), false);
});

test('a timeout renders an error with the budget it gave up after', () => {
  const model = createRunModel({
    config: defaultConfig('tic_tac_toe', 3, 3),
    phase: PHASES.ERROR,
    error: describeRunError('timeout', { timeoutMs: 30000 }),
  });
  const html = renderStudioView(model);

  assert.match(html, /Timed out waiting for finality/);
  assert.match(html, /within 30s/);
  assert.match(html, /stopped waiting/);
});

test('the idle, funding and submitting phases all say what is happening', () => {
  const config = defaultConfig('tic_tac_toe', 3, 3);

  for (const phase of [PHASES.IDLE, PHASES.FUNDING, PHASES.SUBMITTING]) {
    const html = renderStudioView(createRunModel({ config, phase, state: null }));
    assert.match(html, new RegExp(`data-run-phase="${phase}"`));
    assert.ok(html.includes(PHASE_COPY[phase].title), `${phase} title missing`);
    assert.ok(html.includes(PHASE_COPY[phase].detail), `${phase} detail missing`);
  }

  assert.match(renderStudioView(createRunModel({ config, phase: PHASES.IDLE })), /no confirmed state yet/);
});

test('a settled run reports elapsed time, reads and the transaction hash', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-win')));

  assert.match(html, /data-testid="run-meta"[^>]*>settled in 6\.4s after 4 reads · tx 3f9c1d2a/);
});

test('the component accepts a live state object of the same shape, not only fixtures', () => {
  const config = defaultConfig('connect_four', 5, 4, { winLength: 3, marks: { 1: 'R', 2: 'Y' } });
  const cells = new Array(20).fill(0);
  cells[5] = 1; cells[6] = 1; cells[7] = 1;

  // Exactly the contract's GameState shape, passed straight through.
  const liveState = { cells, status: 1, is_x_turn: false, move_count: 5 };
  const model = createRunModel({ config, state: liveState, previousState: liveState, phase: PHASES.SETTLED });
  const html = renderStudioView(model);

  assert.equal(count(html, 'data-cell-index='), 20);
  assert.match(html, /data-cols="5" data-rows="4"/);
  assert.match(html, />R wins</);
  assert.equal(count(html, 'data-winning="true"'), 3);
  assert.match(html, /5×4 · move 5 · Y to move/);
});

test('the view is a single scoped fragment that a host can mount', () => {
  const html = renderStudioView(modelFromFixture(loadFixture('3x3-win')));

  assert.match(html, /^<section class="cougr-studio"/);
  assert.match(html, /<\/section>$/);
  assert.equal(count(html, '<section'), 1);

  const styles = studioStyles();
  assert.match(styles, /^\.cougr-studio/);
  assert.equal(styles.includes('<'), false, 'styles must not contain stray markup');
  assert.match(styles, /prefers-reduced-motion/);

  const withoutStyles = renderStudioView(modelFromFixture(loadFixture('3x3-win')), { inlineStyles: false });
  assert.equal(withoutStyles.includes('@keyframes cougr-studio-spin'), false);
  // The only <style> left is the board's font declaration.
  assert.equal(count(withoutStyles, '<style>'), 1);
});

test('the board never exceeds its pixel budget on either axis', () => {
  for (const [width, height] of [[3, 3], [7, 6], [64, 64]]) {
    const config = defaultConfig('game', width, height, { winLength: Math.min(width, height) });
    const html = renderStudioView(createRunModel({ config, state: null }));
    const cell = Math.floor(BOARD_MAX_PX / Math.max(width, height));
    assert.match(html, new RegExp(`viewBox="0 0 ${cell * width} ${cell * height}"`));
  }
});
