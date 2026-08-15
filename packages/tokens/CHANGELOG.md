# Changelog

All notable changes to `cougr-tokens`. This package versions independently of `cougr-core`, per
the policy in [README.md](./README.md#versioning-policy).

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
