/**
 * Tic-tac-toe SVG renderer.
 *
 * Input state shape (from examples/tic_tac_toe/src/lib.rs :: GameState):
 *   cells:      number[9]   — 0=empty, 1=X, 2=O  (row-major, index 0=top-left)
 *   status:     number      — 0=in progress, 1=X wins, 2=O wins, 3=draw
 *   is_x_turn:  boolean
 *   move_count: number
 *
 * Produces a 400×480 SVG (board 400×400 + status bar 80px).
 */

const SIZE = 400;
const CELL = SIZE / 3;       // 133.33…
const PAD = 24;              // padding inside each cell for the X/O

const COLORS = {
  bg: '#0f172a',
  grid: '#334155',
  empty: '#1e293b',
  x_fill: '#f43f5e',
  x_win_fill: '#fda4af',
  o_fill: '#38bdf8',
  o_win_fill: '#93c5fd',
  win_highlight: '#fef08a22',
  status_bar_bg: '#1e293b',
  status_text: '#f1f5f9',
  badge_x: '#f43f5e',
  badge_o: '#38bdf8',
  badge_draw: '#94a3b8',
  badge_ongoing: '#64748b',
  label: '#64748b',
};

/** Calculate which cells are part of the winning line (for highlighting). */
function winningCells(cells) {
  const LINES = [
    [0,1,2],[3,4,5],[6,7,8], // rows
    [0,3,6],[1,4,7],[2,5,8], // cols
    [0,4,8],[2,4,6],         // diagonals
  ];
  for (const [a,b,c] of LINES) {
    if (cells[a] !== 0 && cells[a] === cells[b] && cells[a] === cells[c]) {
      return new Set([a,b,c]);
    }
  }
  return new Set();
}

function renderX(cx, cy, half, isWin) {
  const color = isWin ? COLORS.x_win_fill : COLORS.x_fill;
  const t = half - PAD;
  return `
    <line x1="${cx-t}" y1="${cy-t}" x2="${cx+t}" y2="${cy+t}"
          stroke="${color}" stroke-width="14" stroke-linecap="round"/>
    <line x1="${cx+t}" y1="${cy-t}" x2="${cx-t}" y2="${cy+t}"
          stroke="${color}" stroke-width="14" stroke-linecap="round"/>
  `;
}

function renderO(cx, cy, half, isWin) {
  const color = isWin ? COLORS.o_win_fill : COLORS.o_fill;
  const r = half - PAD - 4;
  return `<circle cx="${cx}" cy="${cy}" r="${r}"
            fill="none" stroke="${color}" stroke-width="12"/>`;
}

function statusLabel(status, isXTurn) {
  if (status === 1) return { text: '✕  wins', color: COLORS.badge_x };
  if (status === 2) return { text: '○  wins', color: COLORS.badge_o };
  if (status === 3) return { text: 'Draw', color: COLORS.badge_draw };
  return { text: isXTurn ? '✕  to move' : '○  to move', color: COLORS.badge_ongoing };
}

export function render(state) {
  const { cells, status, is_x_turn, move_count } = state;
  const won = winningCells(cells);
  const half = CELL / 2;
  const TOTAL_H = SIZE + 80;

  // Background
  let svg = `<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 ${SIZE} ${TOTAL_H}" width="${SIZE}" height="${TOTAL_H}">
  <defs>
    <style>
      text { font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; }
    </style>
  </defs>

  <!-- Background -->
  <rect width="${SIZE}" height="${TOTAL_H}" fill="${COLORS.bg}" rx="12"/>

  <!-- Board grid cells -->
`;

  // Draw cells
  for (let i = 0; i < 9; i++) {
    const row = Math.floor(i / 3);
    const col = i % 3;
    const x = col * CELL;
    const y = row * CELL;
    const cx = x + half;
    const cy = y + half;
    const isWinCell = won.has(i);

    // Cell background
    svg += `  <rect x="${x+2}" y="${y+2}" width="${CELL-4}" height="${CELL-4}"
        fill="${isWinCell ? COLORS.win_highlight : COLORS.empty}" rx="4"/>\n`;

    // Grid lines (right / bottom borders only, skip last)
    if (col < 2) {
      svg += `  <line x1="${x+CELL}" y1="${2}" x2="${x+CELL}" y2="${SIZE-2}"
        stroke="${COLORS.grid}" stroke-width="3"/>\n`;
    }
    if (row < 2 && col === 0) {
      svg += `  <line x1="${2}" y1="${y+CELL}" x2="${SIZE-2}" y2="${y+CELL}"
        stroke="${COLORS.grid}" stroke-width="3"/>\n`;
    }

    // Piece
    if (cells[i] === 1) svg += renderX(cx, cy, half, isWinCell);
    if (cells[i] === 2) svg += renderO(cx, cy, half, isWinCell);
  }

  // Status bar
  const { text: statusText, color: statusColor } = statusLabel(status, is_x_turn);
  const moveLabel = `Move ${move_count}`;

  svg += `
  <!-- Status bar -->
  <rect x="0" y="${SIZE}" width="${SIZE}" height="80" fill="${COLORS.status_bar_bg}" rx="0"/>
  <rect x="0" y="${SIZE}" width="${SIZE}" height="2" fill="${COLORS.grid}"/>

  <!-- Status badge -->
  <rect x="16" y="${SIZE+16}" width="160" height="36" rx="18" fill="${statusColor}22"/>
  <text x="96" y="${SIZE+40}" text-anchor="middle" font-size="15" font-weight="700"
        fill="${statusColor}">${statusText}</text>

  <!-- Move counter -->
  <text x="${SIZE-16}" y="${SIZE+40}" text-anchor="end" font-size="13"
        fill="${COLORS.label}">${moveLabel}</text>

  <!-- Cougr watermark -->
  <text x="${SIZE/2}" y="${SIZE+68}" text-anchor="middle" font-size="10"
        fill="${COLORS.label}" letter-spacing="2">COUGR · TIC-TAC-TOE</text>
`;

  svg += `</svg>`;
  return svg;
}
