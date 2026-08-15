/**
 * Preview theme, resolved from the shared design tokens package.
 *
 * Previews are standalone SVG files consumed as images, so CSS custom
 * properties from a host page never reach them. That is why this module reads
 * literal values from `cougr-tokens` at generation time rather than emitting
 * `var(--color-bg)` into the markup.
 *
 * The package is imported by relative path because this repository has no npm
 * workspace and `tools/preview-gen` is intentionally install-free. A consumer
 * in a separate repository (the documentation site, per
 * docs/strategy/10-repository-strategy.md) pins `cougr-tokens` as a normal
 * dependency instead.
 *
 * To change any color here, edit `packages/tokens/tokens.json`, run
 * `node build.js` in that directory, then regenerate the previews.
 */

import { dark, version as tokensVersion } from '../../packages/tokens/dist/tokens.js';

export { tokensVersion };

/**
 * Brand tokens, dark mode. Previews are dark-surface artifacts so that they sit
 * on either a light or a dark gallery page without a second render pass.
 */
export const BRAND = dark;

/**
 * Values docs/BRAND.md deliberately does not define.
 *
 * The brand palette is a small fixed vocabulary of interface colors; it has no
 * "damage" hue, and inventing one there would widen the palette for a case only
 * the previews have. These stay local and game-semantic. If a status palette is
 * ever added to the brand, these should move into the tokens package.
 */
export const GAME = Object.freeze({
  /** A struck cell. Warm red, chosen to sit with the brand's warm neutrals. */
  hit: '#D2503C',
  /** Marker stroke drawn over a struck cell. */
  hitMark: '#F2A79A',
});

/** Hairlines: grid dividers and board borders, as a stroke plus an opacity. */
export const LINE = Object.freeze({
  stroke: BRAND.colorTextSecondary,
  gridOpacity: 0.22,
  borderOpacity: 0.35,
});

/** The one font declaration every renderer emits. */
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
