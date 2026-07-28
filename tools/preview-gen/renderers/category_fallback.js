/**
 * Category fallback renderer.
 *
 * For games where board/grid state visualization is not applicable
 * (Snake, Asteroids, Pong, etc.), this renderer generates a branded
 * category card with an icon, game name, and category badge.
 *
 * Input:
 *   game:     string  — game slug, e.g. "snake"
 *   category: string  — e.g. "Arcade"
 *   color:    string  — hex accent color
 *   icon:     string  — emoji icon
 *
 * Produces a 400×300 SVG.
 */

const W = 400;
const H = 300;

function slugToTitle(slug) {
  return slug.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

export function render({ game, category, color, icon }) {
  const title = slugToTitle(game);

  // Darken version of color for gradient stop
  const darkBg = '#0f172a';
  const cardBg = '#1e293b';
  const border = '#334155';
  const textMuted = '#64748b';
  const textPrimary = '#f1f5f9';

  return `<svg xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 ${W} ${H}" width="${W}" height="${H}">
  <defs>
    <style>text { font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; }</style>
    <radialGradient id="glow" cx="50%" cy="45%" r="50%">
      <stop offset="0%" stop-color="${color}" stop-opacity="0.12"/>
      <stop offset="100%" stop-color="${darkBg}" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="border_grad" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="${color}" stop-opacity="0.6"/>
      <stop offset="100%" stop-color="${border}" stop-opacity="0.4"/>
    </linearGradient>
  </defs>

  <!-- Background -->
  <rect width="${W}" height="${H}" fill="${darkBg}" rx="12"/>
  <rect x="1" y="1" width="${W-2}" height="${H-2}" fill="url(#glow)" rx="11"/>

  <!-- Card -->
  <rect x="32" y="32" width="${W-64}" height="${H-64}" fill="${cardBg}" rx="16"
        stroke="url(#border_grad)" stroke-width="1.5"/>

  <!-- Decorative corner accents -->
  <line x1="32" y1="60" x2="56" y2="60" stroke="${color}" stroke-width="2" opacity="0.5"/>
  <line x1="56" y1="32" x2="56" y2="56" stroke="${color}" stroke-width="2" opacity="0.5"/>
  <line x1="${W-32}" y1="60" x2="${W-56}" y2="60" stroke="${color}" stroke-width="2" opacity="0.5"/>
  <line x1="${W-56}" y1="32" x2="${W-56}" y2="56" stroke="${color}" stroke-width="2" opacity="0.5"/>

  <!-- Icon -->
  <text x="${W/2}" y="${H/2 - 18}" text-anchor="middle" font-size="56">${icon}</text>

  <!-- Game title -->
  <text x="${W/2}" y="${H/2 + 32}" text-anchor="middle" font-size="22" font-weight="700"
        fill="${textPrimary}" letter-spacing="0.5">${title}</text>

  <!-- Category badge -->
  <rect x="${W/2 - 50}" y="${H/2 + 46}" width="100" height="24" rx="12" fill="${color}22"/>
  <text x="${W/2}" y="${H/2 + 63}" text-anchor="middle" font-size="12" font-weight="600"
        fill="${color}">${category}</text>

  <!-- "No board preview" note -->
  <text x="${W/2}" y="${H - 44}" text-anchor="middle" font-size="11" fill="${textMuted}">
    Real-time game — no static board state
  </text>

  <!-- Cougr watermark -->
  <text x="${W/2}" y="${H - 26}" text-anchor="middle" font-size="10"
        fill="${textMuted}" letter-spacing="2">COUGR EXAMPLE</text>
</svg>`;
}
