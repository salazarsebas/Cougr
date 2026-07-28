# Cougr Preview Generator

`tools/preview-gen` is a lightweight Node.js CLI that generates SVG preview images for Cougr contract-only examples. Previews are driven by actual scenario/test state data, not hand-drawn art or mocked screenshots.

## Design Rationale

Most Cougr examples are headless smart contracts with no frontend. A showcase gallery needs a visual preview per entry. Rather than faking screenshots or hand-drawing images, this tool extracts final-state data from each example's existing test assertions and renders it as a deterministic, reproducible SVG.

**Board/grid games** (Tic-Tac-Toe, Checkers, Battleship, …) get a full state renderer.  
**Real-time arcade games** (Snake, Asteroids, Pong, …) get a branded category-icon fallback, explicitly labeled as such — not faked.

---

## Requirements

- Node.js ≥ 18 (ESM modules, no npm dependencies)

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
- Dark background (`#0f172a`)
- Board area with subtle grid lines (`#334155`)
- Status bar at the bottom (64–80px) showing turn/result
- `COUGR · GAME_NAME` watermark in the status bar
- No external image references or fonts (SVG must render without network access)

### Step 4 — Register the renderer

In `generate.js`, add an import and register the renderer:

```js
import { render as renderYourGame } from './renderers/your_game.js';

const RENDERERS = {
  // ... existing ...
  your_game: renderYourGame,   // ← add this
};
```

If your game is a real-time/physics game with no meaningful static state, add it to `FALLBACK_GAMES` instead:

```js
const FALLBACK_GAMES = {
  your_game: { category: 'Arcade', color: '#6366f1', icon: '🚀' },
};
```

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

## Palette Reference

| Token | Hex | Use |
|---|---|---|
| `bg` | `#0f172a` | SVG background |
| `card` | `#1e293b` | Cell / status bar background |
| `grid_line` | `#334155` | Board grid dividers |
| `p1_accent` | `#f43f5e` | Player 1 / X pieces |
| `p2_accent` | `#38bdf8` | Player 2 / O pieces |
| `hit` | `#ef4444` | Battleship hit cell |
| `miss` | `#334155` | Battleship miss cell |
| `win_highlight` | `#fef08a22` | Winning-line cell tint |
| `text_muted` | `#64748b` | Labels, watermark |
| `text_primary` | `#f1f5f9` | Main status text |
