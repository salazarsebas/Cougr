import { BRAND, FONT_STYLE, TINT, px } from '../theme.js';

const WIDTH = 520;
const HEIGHT = 310;

function matchCard(x, y, title, players, winner) {
  const cardWidth = 148;
  const cardHeight = 84;
  const rows = players.map((player, index) => {
    const rowY = y + 42 + index * 24;
    const isWinner = player === winner;
    return `
      <rect x="${x + 8}" y="${rowY - 16}" width="${cardWidth - 16}" height="21"
            rx="${px(BRAND.radiusSm)}" fill="${isWinner ? `${BRAND.colorTierStable}${TINT}` : BRAND.colorBg}"/>
      <text x="${x + 16}" y="${rowY}" font-size="12" font-weight="${isWinner ? 700 : 400}"
            fill="${isWinner ? BRAND.colorTierStable : BRAND.colorTextSecondary}">${player}</text>
      ${isWinner ? `<text x="${x + cardWidth - 18}" y="${rowY}" text-anchor="end" font-size="10" fill="${BRAND.colorTierStable}">WIN</text>` : ''}
    `;
  }).join('');

  return `
    <rect x="${x}" y="${y}" width="${cardWidth}" height="${cardHeight}"
          rx="${px(BRAND.radiusMd)}" fill="${BRAND.colorSurface}"
          stroke="${BRAND.colorTextSecondary}" stroke-opacity="0.24"/>
    <text x="${x + 12}" y="${y + 22}" font-size="11" font-weight="700"
          fill="${BRAND.colorText}">${title}</text>
    ${rows}
  `;
}

export function render(state) {
  const leftX = 24;
  const finalX = 348;
  const firstY = 82;
  const secondY = 204;
  const finalY = 143;
  const firstCenterA = firstY + 42;
  const firstCenterB = secondY + 42;
  const finalCenter = finalY + 42;

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${HEIGHT}" width="${WIDTH}" height="${HEIGHT}">
  <defs><style>${FONT_STYLE}</style></defs>
  <rect width="${WIDTH}" height="${HEIGHT}" fill="${BRAND.colorBg}" rx="${px(BRAND.radiusLg)}"/>
  <text x="24" y="34" font-size="16" font-weight="700" fill="${BRAND.colorText}">Staked tournament</text>
  <text x="24" y="54" font-size="11" fill="${BRAND.colorTextSecondary}">Four players · single elimination · per-match escrow</text>
  <path d="M172 ${firstCenterA} H220 V${finalCenter} H${finalX}" fill="none" stroke="${BRAND.colorTextSecondary}" stroke-opacity="0.5" stroke-width="2"/>
  <path d="M172 ${firstCenterB} H220 V${finalCenter} H${finalX}" fill="none" stroke="${BRAND.colorTextSecondary}" stroke-opacity="0.5" stroke-width="2"/>
  ${matchCard(leftX, firstY, 'SEMIFINAL 1', state.semifinals[0].players, state.semifinals[0].winner)}
  ${matchCard(leftX, secondY, 'SEMIFINAL 2', state.semifinals[1].players, state.semifinals[1].winner)}
  ${matchCard(finalX, finalY, 'FINAL', state.final.players, state.final.winner)}
  <text x="${WIDTH - 24}" y="${HEIGHT - 18}" text-anchor="end" font-size="10" fill="${BRAND.colorTierStable}">CHAMPION: ${state.champion}</text>
</svg>`;
}