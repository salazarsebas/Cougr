#!/usr/bin/env node
/**
 * Render every recorded fixture to disk as an HTML page and a bare SVG board.
 *
 * This is the human check that pairs with the tests: the tests assert the
 * markup, this writes artifacts a reviewer can open. Output goes to `out/`,
 * which is gitignored — it is a build product, not a source file.
 *
 * Usage: node tools/render-fixtures.js
 */

import fs from 'node:fs';
import path from 'node:path';

import { renderBoard } from '../src/board.js';
import { renderStudioView } from '../src/view.js';
import { listFixtures, loadFixture, modelFromFixture, STUDIO_DIR } from './fixtures.js';

const OUT_DIR = path.join(STUDIO_DIR, 'out');

const PAGE = (title, fragment) => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title} · Cougr studio fixtures</title>
<style>body { margin: 0; padding: 24px; background: #0b0907; }</style>
</head>
<body>
${fragment}
</body>
</html>
`;

function main() {
  fs.mkdirSync(OUT_DIR, { recursive: true });

  const ids = listFixtures();

  for (const id of ids) {
    const fixture = loadFixture(id);
    const model = modelFromFixture(fixture);

    const svg = renderBoard(fixture.config, fixture.state);
    const html = renderStudioView(model);

    fs.writeFileSync(path.join(OUT_DIR, `${id}.svg`), `${svg}\n`, 'utf8');
    fs.writeFileSync(path.join(OUT_DIR, `${id}.html`), PAGE(id, html), 'utf8');

    const cells = fixture.state ? fixture.state.cells.length : 0;
    console.log(`✓ ${id.padEnd(22)} ${model.phase.padEnd(10)} ${cells} cells  out/${id}.html  out/${id}.svg`);
  }

  console.log(`\nWrote ${ids.length * 2} files to ${path.relative(process.cwd(), OUT_DIR)}`);
}

main();
