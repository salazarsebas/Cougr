/**
 * Studio theme, resolved from the shared design tokens package.
 *
 * The board and the run panels are rendered as standalone HTML/SVG fragments,
 * so a host page's CSS custom properties never reach them. That is why this
 * module reads literal values from `cougr-tokens` rather than emitting
 * `var(--color-bg)` into the markup — the same reason `tools/preview-gen`
 * resolves its tokens at generation time.
 *
 * The package is imported by relative path because this repository has no npm
 * workspace and `studio/` is intentionally install-free. A host app in its own
 * repository pins `cougr-tokens` as a normal dependency instead. The built
 * `dist/` is not committed, so run `node ../packages/tokens/build.js` (or
 * `npm run build:tokens`) before importing this module.
 *
 * To change any color here, edit `packages/tokens/tokens.json`, run
 * `node build.js` in that directory, then re-render the fixtures.
 */

import { dark, version as tokensVersion } from '../../packages/tokens/dist/tokens.js';

export { tokensVersion };

/**
 * Studio surfaces are dark by default, matching the showcase and the SVG
 * previews: a board reads as a lit surface and sits on either a light or a
 * dark host page without a second render pass.
 */
export const BRAND = dark;

/**
 * Semantic colors the brand palette deliberately does not define.
 *
 * `docs/BRAND.md` scopes the palette to a small fixed vocabulary and claims
 * red for error states — but it has no error *token*, because error surfaces
 * did not exist when the palette was written. These stay local and semantic
 * until it does. Waiting reuses the Beta tier and settled reuses the Stable
 * tier, so a run's lifecycle reads in the same colors as the maturity model.
 */
export const STUDIO = Object.freeze({
  /** Error state: RPC failure, friendbot failure, timeout. */
  danger: '#D2503C',
  dangerText: '#F2A79A',
  /** Waiting for Soroban finality. */
  waiting: BRAND.colorTierBeta,
  /** The run settled and the board is confirmed. */
  settled: BRAND.colorTierStable,
});

/** Hairlines: grid dividers and panel borders, as a stroke plus an opacity. */
export const LINE = Object.freeze({
  stroke: BRAND.colorTextSecondary,
  gridOpacity: 0.22,
  borderOpacity: 0.35,
});

/** The one font declaration the SVG board emits. */
export const FONT_STYLE = `text { font-family: ${BRAND.fontSans}; }`;

/** Tint suffix for badge fills, matching the existing `${color}22` idiom. */
export const TINT = '22';

/**
 * Strip the unit off a spacing or radius token.
 *
 * SVG 1.1 presentation attributes such as `rx` take a bare number, so `"12px"`
 * has to become `12` before it reaches the markup.
 */
export function px(token) {
  return parseFloat(token);
}

/**
 * Apply an 8-digit-hex alpha to a 6-digit token color.
 *
 * The alternative is `rgba()` with a parsed channel triple, which loses the
 * token string; keeping the token visible in the output is worth the two lines.
 */
export function withAlpha(hex, opacity) {
  const alpha = Math.round(Math.min(Math.max(opacity, 0), 1) * 255)
    .toString(16)
    .padStart(2, '0');
  return `${hex}${alpha}`;
}
