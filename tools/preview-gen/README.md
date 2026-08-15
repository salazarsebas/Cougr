# Cougr Preview Generator

`tools/preview-gen` is a lightweight Node.js CLI that generates SVG preview images for Cougr contract-only examples. Previews are driven by actual scenario/test state data, not hand-drawn art or mocked screenshots.

## Design Rationale

Most Cougr examples are headless smart contracts with no frontend. A showcase gallery needs a visual preview per entry. Rather than faking screenshots or hand-drawing images, this tool extracts final-state data from each example's existing test assertions and renders it as a deterministic, reproducible SVG.

**Board/grid games** (Tic-Tac-Toe, Checkers, Battleship, …) get a full state renderer.  
**Real-time arcade games** (Snake, Asteroids, Pong, …) get a branded category-icon fallback, explicitly labeled as such — not faked.

---

## Requirements

- Node.js ≥ 18 (ESM modules, no npm dependencies)

Renderers read their colors from [`packages/tokens`](../../packages/tokens), whose `dist/` is
generated rather than committed. `generate.js` builds it automatically on first run, so there is no
setup step to remember. To build it yourself:

```bash
cd packages/tokens && node build.js
```

---

## Usage

```bash
cd tools/preview-gen

# Generate a single preview (writes to examples/<game>/preview.svg)
node generate.js tic_tac_toe
node generate.js checkers
node generate.js battleship

# Generate all registered games at once
npm run gen:all
```

---

## Repository Layout

```
tools/preview-gen/
├── generate.js              # Main CLI entry point
├── package.json             # npm scripts + engine requirement
├── README.md                # This file
├── renderers/
│   ├── tic_tac_toe.js       # 3×3 board SVG renderer
│   ├── checkers.js          # 8×8 board SVG renderer
│   ├── battleship.js        # Dual 10×10 attack-grid renderer
│   └── category_fallback.js # Branded icon card for non-grid games
└── states/
    ├── tic_tac_toe.json     # Final state extracted from test_x_wins_row_top
    ├── checkers.json        # Opening position from initial_board() + test assertions
    └── battleship.json      # Mid-game attack grid from reveal_cell tests
```

---

## Adding a Preview for a New Example

Follow these steps when adding a canonical board/grid example to the gallery.

### Step 1 — Identify the representative state

Open the example's `src/test.rs` and find the most illustrative terminal state. Good candidates:

- A **winning** state (the test that asserts `status == 1` or similar)
- The **opening position** for a complex piece-based game
- A **mid-game** state that shows the core mechanic (e.g. attacks/hits for Battleship)

**Do not invent data**. Every value in the JSON must be traceable to a specific test assertion.

### Step 2 — Create `states/<your_game>.json`

```json
{
  "_source": "examples/<your_game>/src/test.rs :: <test_fn_name> (lines N-M)",
  "_description": "One sentence: what game state this represents.",
  "_move_sequence": [ ... ],   // optional: list moves that produce this state
  "field1": ...,               // game-specific state fields
  "field2": ...
}
```

Include the `_source` field so anyone reading the file can trace back to the original test.

### Step 3 — Create `renderers/<your_game>.js`

```js
/**
 * <GameName> SVG renderer.
 *
 * Input state shape:
 *   field1: type  — description
 *   ...
 *
 * Produces a WIDTHxHEIGHT SVG.
 */
export function render(state) {
  // Build SVG string using template literals.
  // Use COLORS object for a consistent dark-mode palette.
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" ...>`;
  // ... draw board, pieces, status bar, Cougr watermark
  svg += `</svg>`;
  return svg;
}
```

**SVG design guidelines:**
- Dark background (`BRAND.colorBg`)
- Board area with subtle grid lines (`LINE.stroke` at `LINE.gridOpacity`)
- Status bar at the bottom (64–80px) showing turn/result
- `COUGR · GAME_NAME` watermark in the status bar
- No external image references or fonts (SVG must render without network access)

### Step 4 — Register the renderer

In `generate.js`, register the renderer by path. It is imported on demand, after the design tokens
it reads have been built:

```js
const RENDERERS = {
  // ... existing ...
  your_game: './renderers/your_game.js',   // ← add this
};
```

If your game is a real-time/physics game with no meaningful static state, add it to `FALLBACK_GAMES` instead:

```js
const FALLBACK_GAMES = {
  your_game: { category: 'Arcade', icon: '🚀' },
};
```

Fallback cards take their accent from the brand tokens, so there is no per-game color to pick.

### Step 5 — Run and verify

```bash
node generate.js your_game
```

Open `examples/your_game/preview.svg` in a browser. Confirm:
- The state matches the test assertion you cited in `_source`
- The image is legible at 400–520px wide
- The status bar correctly reflects the game outcome

### Step 6 — Check in

```bash
git add \
  examples/your_game/preview.svg \
  tools/preview-gen/states/your_game.json \
  tools/preview-gen/renderers/your_game.js \
  tools/preview-gen/generate.js
```

Update the `Preview` column in `examples/README.md` for your example.

---

## When to Use the Category Fallback

Use `category_fallback` (not a board renderer) when:

- The game has no meaningful static board state (Snake, Asteroids, Pong, Flappy Bird)
- The game state at any single moment doesn't convey how the game works
- Rendering a fake "representative" frame would be misleading

The fallback is explicitly labeled "Real-time game — no static board state" so gallery visitors understand why no gameplay preview is shown.

---

## Palette

Renderers do not define their own colors. `theme.js` resolves them from
[`packages/tokens`](../../packages/tokens), the shared design token package that also feeds the
documentation site, so the showcase and the docs cannot drift apart. The values themselves are
specified in [docs/BRAND.md](../../docs/BRAND.md).

Import `theme.js` rather than writing a hex literal:

```js
import { BRAND, LINE, FONT_STYLE, TINT, px } from '../theme.js';

const COLORS = {
  bg: BRAND.colorBg,
  card: BRAND.colorSurface,
  p1: BRAND.colorPrimary,
  p2: BRAND.colorAccent,
  label: BRAND.colorTextSecondary,
};
```

| Export | Use |
|---|---|
| `BRAND` | Brand tokens in dark mode. Previews are dark-surface artifacts so they sit on either a light or a dark gallery page. |
| `GAME` | The few game-semantic values the brand palette deliberately does not define, currently the battleship hit colors. |
| `LINE` | Stroke color plus grid and border opacities for hairlines. |
| `FONT_STYLE` | The `<style>` body every renderer emits, using the brand sans stack. |
| `TINT` | Alpha suffix for badge fills, as in `${statusColor}${TINT}`. |
| `px(token)` | Strips the unit off a spacing or radius token for an SVG attribute. |

To change a preview color, edit `packages/tokens/tokens.json`, run `node build.js` there, then
regenerate the previews with `npm run gen:all`. Do not edit a hex value in a renderer.

Renderers are imported after the tokens are built, so a renderer can read `BRAND` at module load.
Register new ones by path in `RENDERERS`, not by a static import.
