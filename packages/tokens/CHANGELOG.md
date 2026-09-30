# Changelog

All notable changes to `cougr-tokens`. This package versions independently of `cougr-core`, per
the policy in [README.md](./README.md#versioning-policy).

## 1.1.0

### Added

- **`color-danger`**: error and danger token (`#CA4D3A` light, `#F2A79A` dark), promoting the
  RPC- and friendbot-failure colors from Studio to the shared brand palette with 4.53:1 light and
  9.71:1 dark contrast measurements meeting WCAG AA
- **`color-on-danger`**: on-danger text companion (`#FFFFFF` light, `#14100D` dark) for text
  and iconography on danger surfaces, with 4.53:1 light and 9.71:1 dark contrast on `color-danger`

## 1.0.0

### Added

- **`tokens.json`**: the token source of truth, encoding every value defined in
  [docs/BRAND.md](../../docs/BRAND.md): four neutrals, primary and accent, three maturity-tier
  colors, two font stacks, an eight-step spacing scale, four radii, and the four fixed logo tones
- **`dist/tokens.css`**: built CSS custom properties with light and dark sets, switched by
  `prefers-color-scheme` and overridable with a `data-theme` attribute on the root element
- **`dist/tokens.js`**: built ESM module exporting `light`, `dark`, `tokens`, `theme(mode)`,
  and `version`, for consumers that need literal values at build time
- **`build.js`**: zero-dependency transform. `dist/` is generated rather than committed, produced
  by `npm run build` and by the `prepare` script on install and publish. `--check` writes nothing
  and fails when `tokens.json` has drifted from `docs/BRAND.md`, the source of truth
- **CI**: a `Design Tokens` workflow that verifies the source against `docs/BRAND.md`, builds,
  loads the built module, and asserts `dist/` is not tracked
