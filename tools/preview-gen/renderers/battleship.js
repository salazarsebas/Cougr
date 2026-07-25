/**
 * Battleship SVG renderer.
 *
 * Battleship uses a commit-reveal scheme: each player commits a board,
 * then attacks. The publicly visible state is the *attack grid* —
 * which cells have been fired at and whether they were hits or misses.
 *
 * Input state shape (from examples/battleship/src/lib.rs :: GameState):
 *   grid:           Record<number, "Hit"|"Miss"|"Unknown">
 *                   — map from cell index (0-99) to result
 *                   — cells not present = Unknown (not yet attacked)
 *   grid_size:      number  — default 10
 *   label:          string  — e.g. "Player A's attack grid"
 *   ship_remaining: number  — ships remaining on the target board
 *   total_ships:    number  — 17 (5+4+3+3+2)
 *   phase:          "Setup" | "Attack" | "Finished"
 *   turn:           "A" | "B"
 *   winner:         null | "A" | "B"
 *
 * Produces a 520×640 SVG (two grids side by side + legend + status bar).
 * Since battleship has two attack grids (A attacks B, B attacks A),
 * we render *both* grids stacked, using state.grid_a and state.grid_b.
 */

const GRID = 10;
const CELL_SZ = 36;         // each cell is 36×36px
const BOARD_W = GRID * CELL_SZ;   // 360
const SVG_W = BOARD_W + 80;       // 440 — extra for labels
const BOARD_H = GRID * CELL_SZ;   // 360
const GAP = 24;                    // gap between the two boards
const STATUS_H = 72;
const SVG_H = 2 * BOARD_H + GAP + STATUS_H + 60;  // header space

const COLORS = {
  bg: '#0f172a',
  sea: '#0c4a6e',            // empty/unknown cell
  hit: '#ef4444',            // hit cell fill
  hit_stroke: '#fca5a5',
  miss: '#334155',           // miss cell fill
  miss_mark: '#94a3b8',
  grid_line: '#1e3a5f',
  label_axis: '#64748b',
  header_text: '#f1f5f9',
  header_a: '#f43f5e',
  header_b: '#38bdf8',
  status_bar: '#1e293b',
  divider: '#475569',
  badge_a: '#f43f5e',
  badge_b: '#38bdf8',
  badge_finished: '#22c55e',
  badge_setup: '#f59e0b',
  badge_attack: '#38bdf8',
  text_muted: '#64748b',
};

const COL_LABELS = ['A','B','C','D','E','F','G','H','I','J'];

function renderGrid(cells, offsetX, offsetY, label, labelColor) {
  let out = '';

  // Label
  out += `<text x="${offsetX + BOARD_W/2}" y="${offsetY - 8}" text-anchor="middle"
    font-size="12" font-weight="600" fill="${labelColor}">${label}</text>`;

  // Column labels (A-J)
  for (let c = 0; c < GRID; c++) {
    out += `<text x="${offsetX + c*CELL_SZ + CELL_SZ/2}" y="${offsetY - 24}"
      text-anchor="middle" font-size="10" fill="${COLORS.label_axis}">${COL_LABELS[c]}</text>`;
  }

  // Row labels (1-10)
  for (let r = 0; r < GRID; r++) {
    out += `<text x="${offsetX - 10}" y="${offsetY + r*CELL_SZ + CELL_SZ/2 + 4}"
      text-anchor="end" font-size="10" fill="${COLORS.label_axis}">${r+1}</text>`;
  }

  for (let r = 0; r < GRID; r++) {
    for (let c = 0; c < GRID; c++) {
      const idx = r * GRID + c;
      const x = offsetX + c * CELL_SZ;
      const y = offsetY + r * CELL_SZ;
      const result = cells[idx] ?? 'Unknown';

      // Cell background
      let fill = COLORS.sea;
      if (result === 'Hit') fill = COLORS.hit;
      else if (result === 'Miss') fill = COLORS.miss;

      out += `<rect x="${x+0.5}" y="${y+0.5}" width="${CELL_SZ-1}" height="${CELL_SZ-1}"
        fill="${fill}" rx="2"/>`;

      // Hit marker: X
      if (result === 'Hit') {
        const pad = 8;
        out += `<line x1="${x+pad}" y1="${y+pad}" x2="${x+CELL_SZ-pad}" y2="${y+CELL_SZ-pad}"
          stroke="${COLORS.hit_stroke}" stroke-width="2.5" stroke-linecap="round"/>
        <line x1="${x+CELL_SZ-pad}" y1="${y+pad}" x2="${x+pad}" y2="${y+CELL_SZ-pad}"
          stroke="${COLORS.hit_stroke}" stroke-width="2.5" stroke-linecap="round"/>`;
      }

      // Miss marker: circle dot
      if (result === 'Miss') {
        out += `<circle cx="${x+CELL_SZ/2}" cy="${y+CELL_SZ/2}" r="4"
          fill="${COLORS.miss_mark}" opacity="0.7"/>`;
      }
    }
  }

  // Grid lines
  for (let i = 0; i <= GRID; i++) {
    out += `<line x1="${offsetX + i*CELL_SZ}" y1="${offsetY}"
      x2="${offsetX + i*CELL_SZ}" y2="${offsetY + BOARD_H}"
      stroke="${COLORS.grid_line}" stroke-width="0.75"/>`;
    out += `<line x1="${offsetX}" y1="${offsetY + i*CELL_SZ}"
      x2="${offsetX + BOARD_W}" y2="${offsetY + i*CELL_SZ}"
      stroke="${COLORS.grid_line}" stroke-width="0.75"/>`;
  }

  // Board border
  out += `<rect x="${offsetX}" y="${offsetY}" width="${BOARD_W}" height="${BOARD_H}"
    fill="none" stroke="${COLORS.divider}" stroke-width="1.5" rx="2"/>`;

  return out;
}

function countResults(cells) {
  let hits = 0, misses = 0;
  for (const v of Object.values(cells)) {
    if (v === 'Hit') hits++;
    else if (v === 'Miss') misses++;
  }
  return { hits, misses };
}

function phaseColor(phase) {
  if (phase === 'Setup') return COLORS.badge_setup;
  if (phase === 'Attack') return COLORS.badge_attack;
  return COLORS.badge_finished;
}

export function render(state) {
  const {
    grid_a = {}, grid_b = {},
    ship_remaining_a = 17, ship_remaining_b = 17, total_ships = 17,
    phase = 'Attack', turn = 'A', winner = null,
  } = state;

  const countA = countResults(grid_a);
  const countB = countResults(grid_b);

  const MARGIN_LEFT = 52;    // space for row labels
  const HEADER_TOP = 48;     // top offset for board A

  const boardATop = HEADER_TOP;
  const boardBTop = boardATop + BOARD_H + GAP + 32; // +32 for B's label
  const totalSvgH = boardBTop + BOARD_H + STATUS_H + 16;

  const statusText = winner
    ? `Player ${winner} wins!`
    : phase === 'Setup'
      ? 'Setup phase — awaiting commitments'
      : `Player ${turn}'s turn to attack`;
  const statusColor = winner
    ? COLORS.badge_finished
    : phaseColor(phase);

  let svg = `<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 ${SVG_W} ${totalSvgH}" width="${SVG_W}" height="${totalSvgH}">
  <defs>
    <style>text { font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; }</style>
  </defs>

  <!-- Background -->
  <rect width="${SVG_W}" height="${totalSvgH}" fill="${COLORS.bg}" rx="12"/>

  <!-- Grid A: Player A's attacks on B's fleet -->
`;

  svg += renderGrid(grid_a, MARGIN_LEFT, boardATop, "Player A attacks →", COLORS.header_a);

  // Stats for grid A
  const aHits = countA.hits;
  const aMisses = countA.misses;
  svg += `<text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardATop + 18}" font-size="10" fill="${COLORS.header_a}">
    Hits: ${aHits}</text>
  <text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardATop + 33}" font-size="10" fill="${COLORS.miss_mark}">
    Miss: ${aMisses}</text>
  <text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardATop + 52}" font-size="10" fill="${COLORS.text_muted}">
    Ships: ${ship_remaining_b}/${total_ships}</text>`;

  svg += `\n  <!-- Grid B: Player B's attacks on A's fleet -->\n`;
  svg += renderGrid(grid_b, MARGIN_LEFT, boardBTop, "Player B attacks →", COLORS.header_b);

  // Stats for grid B
  const bHits = countB.hits;
  const bMisses = countB.misses;
  svg += `<text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardBTop + 18}" font-size="10" fill="${COLORS.header_b}">
    Hits: ${bHits}</text>
  <text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardBTop + 33}" font-size="10" fill="${COLORS.miss_mark}">
    Miss: ${bMisses}</text>
  <text x="${MARGIN_LEFT + BOARD_W + 8}" y="${boardBTop + 52}" font-size="10" fill="${COLORS.text_muted}">
    Ships: ${ship_remaining_a}/${total_ships}</text>`;

  // Status bar
  const statusBarY = boardBTop + BOARD_H + 12;
  svg += `
  <!-- Status bar -->
  <rect x="0" y="${statusBarY}" width="${SVG_W}" height="${STATUS_H}" fill="${COLORS.status_bar}" rx="0"/>
  <rect x="0" y="${statusBarY}" width="${SVG_W}" height="1.5" fill="${COLORS.divider}"/>

  <!-- Status badge -->
  <rect x="12" y="${statusBarY+14}" width="${Math.min(statusText.length * 9 + 24, SVG_W - 24)}" height="30" rx="15"
        fill="${statusColor}22"/>
  <text x="${12 + Math.min(statusText.length * 9 + 24, SVG_W - 24)/2}"
        y="${statusBarY+34}" text-anchor="middle" font-size="13" font-weight="600"
        fill="${statusColor}">${statusText}</text>

  <!-- Watermark -->
  <text x="${SVG_W/2}" y="${statusBarY + STATUS_H - 8}" text-anchor="middle" font-size="9"
        fill="${COLORS.text_muted}" letter-spacing="2">COUGR · BATTLESHIP · COMMIT-REVEAL</text>
`;

  svg += `</svg>`;
  return svg;
}
