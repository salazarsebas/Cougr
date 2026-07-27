/**
 * Checkers SVG renderer.
 *
 * Input state shape (from examples/checkers/src/components.rs :: BoardComponent):
 *   cells:          number[64]  — row-major 8×8
 *                   0=empty, 1=P1 man, -1=P2 man, 2=P1 king, -2=P2 king
 *   current_player: number      — 1 or 2
 *   move_number:    number
 *   status:         "Active" | "Finished"
 *   winner:         number      — 0=none, 1=P1, 2=P2
 *
 * Board layout:
 *   P1 starts on rows 0–2 (top), plays on dark squares (row+col odd).
 *   P2 starts on rows 5–7 (bottom), plays on dark squares.
 *
 * Produces a 480×560 SVG (8×8 board 480×480 + status bar 80px).
 */

const COLS = 8;
const SIZE = 480;
const CELL = SIZE / COLS;   // 60
const TOTAL_H = SIZE + 80;

const COLORS = {
  bg: '#0f172a',
  light_sq: '#1e293b',
  dark_sq: '#334155',
  p1_piece: '#f43f5e',      // rose-500
  p1_king: '#fda4af',       // rose-300
  p1_stroke: '#9f1239',
  p2_piece: '#38bdf8',      // sky-400
  p2_king: '#93c5fd',       // blue-300
  p2_stroke: '#0369a1',
  crown: '#fbbf24',
  status_bar: '#1e293b',
  divider: '#475569',
  text_primary: '#f1f5f9',
  text_muted: '#64748b',
  badge_p1: '#f43f5e',
  badge_p2: '#38bdf8',
  badge_draw: '#94a3b8',
  label_p1: '#fda4af',
  label_p2: '#7dd3fc',
};

function cellAt(cells, row, col) {
  return cells[row * COLS + col] ?? 0;
}

function isDark(row, col) {
  return (row + col) % 2 === 1;
}

function renderPiece(x, y, value) {
  const cx = x + CELL / 2;
  const cy = y + CELL / 2;
  const r = CELL / 2 - 6;
  const isKing = Math.abs(value) === 2;
  const isP1 = value > 0;

  const fill = isKing
    ? (isP1 ? COLORS.p1_king : COLORS.p2_king)
    : (isP1 ? COLORS.p1_piece : COLORS.p2_piece);
  const stroke = isP1 ? COLORS.p1_stroke : COLORS.p2_stroke;

  let out = '';

  // Drop shadow
  out += `<circle cx="${cx+1}" cy="${cy+2}" r="${r}" fill="#00000044"/>`;
  // Main piece
  out += `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="2"/>`;
  // Inner ring highlight
  out += `<circle cx="${cx}" cy="${cy}" r="${r*0.6}" fill="none" stroke="${fill}" stroke-width="1.5" opacity="0.5"/>`;

  // King crown indicator — a smaller gold circle in the center
  if (isKing) {
    out += `<circle cx="${cx}" cy="${cy}" r="${r*0.3}" fill="${COLORS.crown}" opacity="0.9"/>`;
  }

  return out;
}

function pieceCount(cells) {
  let p1 = 0, p2 = 0;
  for (const v of cells) {
    if (v > 0) p1++;
    else if (v < 0) p2++;
  }
  return { p1, p2 };
}

function statusLabel(state) {
  if (state.status === 'Finished') {
    if (state.winner === 1) return { text: 'Player 1 wins', color: COLORS.badge_p1 };
    if (state.winner === 2) return { text: 'Player 2 wins', color: COLORS.badge_p2 };
    return { text: 'Draw', color: COLORS.badge_draw };
  }
  return {
    text: `Player ${state.current_player} to move`,
    color: state.current_player === 1 ? COLORS.badge_p1 : COLORS.badge_p2,
  };
}

export function render(state) {
  const { cells, move_number } = state;
  const { p1, p2 } = pieceCount(cells);
  const { text: statusText, color: statusColor } = statusLabel(state);

  let svg = `<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 ${SIZE} ${TOTAL_H}" width="${SIZE}" height="${TOTAL_H}">
  <defs>
    <style>text { font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; }</style>
  </defs>

  <!-- Background -->
  <rect width="${SIZE}" height="${TOTAL_H}" fill="${COLORS.bg}" rx="12"/>

  <!-- Board squares -->
`;

  for (let row = 0; row < COLS; row++) {
    for (let col = 0; col < COLS; col++) {
      const x = col * CELL;
      const y = row * CELL;
      const dark = isDark(row, col);
      svg += `  <rect x="${x}" y="${y}" width="${CELL}" height="${CELL}" fill="${dark ? COLORS.dark_sq : COLORS.light_sq}"/>\n`;

      // Piece
      const v = cellAt(cells, row, col);
      if (v !== 0) {
        svg += renderPiece(x, y, v);
      }
    }
  }

  // Row/col coordinate labels (a–h, 1–8) outside the board
  for (let i = 0; i < 8; i++) {
    const letter = String.fromCharCode(97 + i); // a-h
    svg += `  <text x="${i*CELL + CELL/2}" y="${SIZE - 3}" text-anchor="middle"
      font-size="9" fill="${COLORS.text_muted}" opacity="0.7">${letter}</text>\n`;
    svg += `  <text x="4" y="${i*CELL + CELL/2 + 4}" text-anchor="start"
      font-size="9" fill="${COLORS.text_muted}" opacity="0.7">${8-i}</text>\n`;
  }

  // Board border
  svg += `  <rect x="0" y="0" width="${SIZE}" height="${SIZE}" fill="none"
    stroke="${COLORS.divider}" stroke-width="1.5" rx="0"/>\n`;

  // Status bar
  svg += `
  <!-- Status bar -->
  <rect x="0" y="${SIZE}" width="${SIZE}" height="80" fill="${COLORS.status_bar}"/>
  <rect x="0" y="${SIZE}" width="${SIZE}" height="1.5" fill="${COLORS.divider}"/>

  <!-- Status badge -->
  <rect x="12" y="${SIZE+14}" width="190" height="32" rx="16" fill="${statusColor}22"/>
  <text x="107" y="${SIZE+35}" text-anchor="middle" font-size="14" font-weight="600"
        fill="${statusColor}">${statusText}</text>

  <!-- Piece counts -->
  <circle cx="${SIZE-110}" cy="${SIZE+30}" r="7" fill="${COLORS.p1_piece}"/>
  <text x="${SIZE-98}" y="${SIZE+35}" font-size="13" fill="${COLORS.label_p1}">${p1} pieces</text>

  <circle cx="${SIZE-50}" cy="${SIZE+30}" r="7" fill="${COLORS.p2_piece}"/>
  <text x="${SIZE-38}" y="${SIZE+35}" font-size="13" fill="${COLORS.label_p2}">${p2}</text>

  <!-- Cougr watermark -->
  <text x="${SIZE/2}" y="${SIZE+65}" text-anchor="middle" font-size="10"
        fill="${COLORS.text_muted}" letter-spacing="2">COUGR · CHECKERS · MOVE ${move_number}</text>
`;

  svg += `</svg>`;
  return svg;
}
