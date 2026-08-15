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

import { BRAND, LINE, FONT_STYLE, TINT, px } from '../theme.js';

const COLS = 8;
const SIZE = 480;
const CELL = SIZE / COLS;   // 60
const TOTAL_H = SIZE + 80;

const COLORS = {
  bg: BRAND.colorBg,
  light_sq: BRAND.colorSurface,
  dark_sq: BRAND.colorBg,
  p1_piece: BRAND.colorPrimary,
  p2_piece: BRAND.colorAccent,
  // Kings keep their side's color and are marked by the crown, rather than by
  // a second tint per side that the brand palette does not define.
  crown: BRAND.logoCream,
  piece_stroke: BRAND.colorBg,
  status_bar: BRAND.colorSurface,
  divider: LINE.stroke,
  text_primary: BRAND.colorText,
  text_muted: BRAND.colorTextSecondary,
  badge_p1: BRAND.colorPrimary,
  badge_p2: BRAND.colorAccent,
  badge_draw: BRAND.colorTextSecondary,
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

  const fill = isP1 ? COLORS.p1_piece : COLORS.p2_piece;

  let out = '';

  // Drop shadow
  out += `<circle cx="${cx+1}" cy="${cy+2}" r="${r}" fill="#00000044"/>`;
  // Main piece
  out += `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${fill}" stroke="${COLORS.piece_stroke}" stroke-width="2"/>`;
  // Inner ring highlight
  out += `<circle cx="${cx}" cy="${cy}" r="${r*0.6}" fill="none" stroke="${COLORS.crown}" stroke-width="1.5" opacity="0.35"/>`;

  // King indicator — a cream disc in the center, legible on either side's color
  if (isKing) {
    out += `<circle cx="${cx}" cy="${cy}" r="${r*0.34}" fill="${COLORS.crown}"/>`;
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
    <style>${FONT_STYLE}</style>
  </defs>

  <!-- Background -->
  <rect width="${SIZE}" height="${TOTAL_H}" fill="${COLORS.bg}" rx="${px(BRAND.radiusLg)}"/>

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
    stroke="${COLORS.divider}" stroke-opacity="${LINE.borderOpacity}" stroke-width="1.5" rx="0"/>\n`;

  // Status bar
  svg += `
  <!-- Status bar -->
  <rect x="0" y="${SIZE}" width="${SIZE}" height="80" fill="${COLORS.status_bar}"/>
  <rect x="0" y="${SIZE}" width="${SIZE}" height="1.5" fill="${COLORS.divider}" fill-opacity="${LINE.borderOpacity}"/>

  <!-- Status badge -->
  <rect x="12" y="${SIZE+14}" width="190" height="32" rx="16" fill="${statusColor}${TINT}"/>
  <text x="107" y="${SIZE+35}" text-anchor="middle" font-size="14" font-weight="600"
        fill="${statusColor}">${statusText}</text>

  <!-- Piece counts -->
  <circle cx="${SIZE-110}" cy="${SIZE+30}" r="7" fill="${COLORS.p1_piece}"/>
  <text x="${SIZE-98}" y="${SIZE+35}" font-size="13" fill="${COLORS.p1_piece}">${p1} pieces</text>

  <circle cx="${SIZE-50}" cy="${SIZE+30}" r="7" fill="${COLORS.p2_piece}"/>
  <text x="${SIZE-38}" y="${SIZE+35}" font-size="13" fill="${COLORS.p2_piece}">${p2}</text>

  <!-- Cougr watermark -->
  <text x="${SIZE/2}" y="${SIZE+65}" text-anchor="middle" font-size="10"
        fill="${COLORS.text_muted}" letter-spacing="2">COUGR · CHECKERS · MOVE ${move_number}</text>
`;

  svg += `</svg>`;
  return svg;
}
