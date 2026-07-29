/**
 * Cougr Preview Generator — main CLI entry point.
 *
 * Usage:
 *   node generate.js <game>
 *
 * Where <game> is one of: tic_tac_toe, checkers, battleship
 *
 * The generator reads a state snapshot from `states/<game>.json`,
 * dispatches to the appropriate renderer, and writes:
 *   ../../examples/<game>/preview.svg
 *
 * State data is manually extracted from the canonical test assertions
 * in each example's `src/test.rs`. See tools/preview-gen/README.md for
 * the full contributor workflow.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Renderer registry — add new games here
import { render as renderTicTacToe } from './renderers/tic_tac_toe.js';
import { render as renderCheckers } from './renderers/checkers.js';
import { render as renderBattleship } from './renderers/battleship.js';
import { render as renderFallback } from './renderers/category_fallback.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const RENDERERS = {
  tic_tac_toe: renderTicTacToe,
  checkers: renderCheckers,
  battleship: renderBattleship,
};

// Games that get the category-icon fallback treatment
const FALLBACK_GAMES = {
  snake: { category: 'Arcade', color: '#22c55e', icon: '🐍' },
  asteroids: { category: 'Arcade', color: '#6366f1', icon: '☄️' },
  pong: { category: 'Arcade', color: '#0ea5e9', icon: '🏓' },
  flappy_bird: { category: 'Arcade', color: '#f59e0b', icon: '🐦' },
};

function main() {
  const game = process.argv[2];

  if (!game) {
    console.error('Usage: node generate.js <game>');
    console.error('Available games:', [...Object.keys(RENDERERS), ...Object.keys(FALLBACK_GAMES)].join(', '));
    process.exit(1);
  }

  // Determine output path
  const outDir = path.resolve(__dirname, '../../examples', game);
  if (!fs.existsSync(outDir)) {
    console.error(`Error: examples/${game}/ does not exist.`);
    process.exit(1);
  }
  const outPath = path.join(outDir, 'preview.svg');

  // Handle fallback games
  if (FALLBACK_GAMES[game]) {
    const svg = renderFallback({ game, ...FALLBACK_GAMES[game] });
    fs.writeFileSync(outPath, svg, 'utf8');
    console.log(`✓ Written fallback preview: ${outPath}`);
    return;
  }

  // Load state JSON
  const statePath = path.resolve(__dirname, 'states', `${game}.json`);
  if (!fs.existsSync(statePath)) {
    console.error(`Error: No state file found at ${statePath}`);
    console.error('  Create one following the instructions in tools/preview-gen/README.md');
    process.exit(1);
  }

  const state = JSON.parse(fs.readFileSync(statePath, 'utf8'));

  // Dispatch to renderer
  const renderer = RENDERERS[game];
  if (!renderer) {
    console.error(`Error: No renderer for "${game}". Add one to renderers/ and register it in generate.js`);
    process.exit(1);
  }

  const svg = renderer(state);
  fs.writeFileSync(outPath, svg, 'utf8');
  console.log(`✓ Written preview: ${outPath}`);
  if (state._source) {
    console.log(`  State source: ${state._source}`);
  }
}

main();
