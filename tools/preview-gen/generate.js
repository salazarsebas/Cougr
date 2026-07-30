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
import { execFileSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const TOKENS_DIR = path.resolve(__dirname, '../../packages/tokens');
const TOKENS_ENTRY = path.join(TOKENS_DIR, 'dist', 'tokens.js');

// Renderer registry — add new games here.
// Loaded on demand, after the design tokens they depend on have been built.
const RENDERERS = {
  tic_tac_toe: './renderers/tic_tac_toe.js',
  checkers: './renderers/checkers.js',
  battleship: './renderers/battleship.js',
};

/**
 * Build the design tokens if they are not built yet.
 *
 * `packages/tokens/dist` is generated rather than committed, so a fresh clone
 * has no token values for the renderers to import. Building here keeps
 * `node generate.js <game>` working on its own, without a separate setup step
 * to remember.
 */
function ensureTokensBuilt() {
  if (fs.existsSync(TOKENS_ENTRY)) return;

  console.log('Design tokens not built yet, building them first...');
  try {
    execFileSync(process.execPath, ['build.js'], { cwd: TOKENS_DIR, stdio: 'inherit' });
  } catch {
    console.error(`Error: failed to build design tokens in ${TOKENS_DIR}.`);
    console.error('  Run `node build.js` there and try again.');
    process.exit(1);
  }
}

// Games that get the category-icon fallback treatment. The accent color comes
// from the shared design tokens, not from a per-game hex here.
const FALLBACK_GAMES = {
  snake: { category: 'Arcade', icon: '🐍' },
  asteroids: { category: 'Arcade', icon: '☄️' },
  pong: { category: 'Arcade', icon: '🏓' },
  flappy_bird: { category: 'Arcade', icon: '🐦' },
};

async function main() {
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

  // Renderers read brand values at module load, so the tokens have to exist
  // before any of them is imported.
  ensureTokensBuilt();

  // Handle fallback games
  if (FALLBACK_GAMES[game]) {
    const { render: renderFallback } = await import('./renderers/category_fallback.js');
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
  const rendererPath = RENDERERS[game];
  if (!rendererPath) {
    console.error(`Error: No renderer for "${game}". Add one to renderers/ and register it in generate.js`);
    process.exit(1);
  }

  const { render } = await import(rendererPath);
  const svg = render(state);
  fs.writeFileSync(outPath, svg, 'utf8');
  console.log(`✓ Written preview: ${outPath}`);
  if (state._source) {
    console.log(`  State source: ${state._source}`);
  }
}

main().catch((error) => {
  console.error('Error:', error.message);
  process.exit(1);
});
