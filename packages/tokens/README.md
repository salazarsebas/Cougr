# cougr-tokens

The single, versioned source for Cougr's design tokens. Every surface that renders Cougr's visual
identity imports these values instead of copying them, so the documentation site and the showcase
cannot drift apart.

The values themselves are specified and justified in [docs/BRAND.md](../../docs/BRAND.md),
including the contrast measurements behind each color pair. This package encodes that document;
it does not extend it.

## What is in here

| Path | Role |
|---|---|
| `tokens.json` | Source of truth. The only file edited by hand. |
| `build.js` | Zero-dependency transform, `tokens.json` to `dist/`. |
| `dist/tokens.css` | Built CSS custom properties, for static HTML/CSS consumers. |
| `dist/tokens.js` | Built ESM module of literal values, for build-time consumers. |

`dist/` is generated and is not committed:

```bash
npm run build          # or: node build.js
```

The `prepare` script runs the same build on `npm install` and before
`npm pack`/`npm publish`, so anyone installing this package gets built output
without running the build themselves, including a consumer in a separate
repository (see
[docs/strategy/10-repository-strategy.md](../../docs/strategy/10-repository-strategy.md)).

`node build.js --check` validates the source and a dry-run build without writing
anything. It fails when `tokens.json` has drifted from `docs/BRAND.md`, which is the drift that
matters once the built output is no longer in version control. CI runs it on any change to this
package or to `docs/BRAND.md`.

## Why two output formats

CSS custom properties are the simpler option and are the right default for anything rendering in a
browser. They are not sufficient on their own, because some consumers need literal values at
generation time rather than at CSS resolution time: anything producing a standalone artifact (an
SVG, a PNG, terminal output) is consumed outside a document, so custom properties declared by a
host page never reach it. The showcase preview generator is the case this package was sized
against. That is the build-time transform need that justifies shipping a package rather than a lone
stylesheet.

Both outputs come from the same source in the same build, so they cannot disagree.

## Using the CSS

```html
<link rel="stylesheet" href="node_modules/cougr-tokens/dist/tokens.css">
```

```css
.card {
  background: var(--color-surface);
  color: var(--color-text);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  font-family: var(--font-sans);
}
```

Theming works in two layers:

- Light is the default, declared on `:root`.
- Dark applies automatically under `@media (prefers-color-scheme: dark)`, unless the document has
  opted out with `data-theme="light"`.
- An explicit `data-theme="light"` or `data-theme="dark"` on the root element always wins, which is
  what a theme toggle sets.

```html
<html data-theme="dark">
```

## Using the JavaScript

```js
import { dark, light, theme, version } from 'cougr-tokens';

dark.colorBg;        // '#14100D'
light.colorPrimary;  // '#8A5A22'
theme('dark').colorTierStable;
```

Token names are the CSS custom property names without the `--` prefix, camel-cased:
`--color-text-secondary` becomes `colorTextSecondary`, `--space-4` becomes `space4`. Values that do
not change between modes (typography, spacing, radius, logo tones) are present in both objects.

## Consuming it

A separate repository depends on the package normally and pins a version, and `prepare` builds
`dist/` during install:

```json
{ "dependencies": { "cougr-tokens": "^1.0.0" } }
```

This repository has no npm workspace, so an in-repo consumer runs this package's build and then
imports the output by relative path. Wire the build into whatever script produces the consumer's
artifacts, so the two cannot be run out of order:

```json
{ "scripts": { "prebuild": "node ../../packages/tokens/build.js" } }
```

## Changing a token

1. Update [docs/BRAND.md](../../docs/BRAND.md) first. It is the source of truth, and it carries the
   contrast measurement that justifies the value.
2. Mirror the change in `tokens.json` and bump `version` there and in `package.json`.
3. Run `node build.js --check` to confirm the two agree, then `node build.js`. There is no built
   output to commit.
4. Add a `CHANGELOG.md` entry.
5. Regenerate anything downstream that bakes token values into committed artifacts.

`node build.js --check` writes nothing and exits non-zero if `tokens.json` has drifted from
`docs/BRAND.md` or fails to build, which is the check to run before opening a pull request. CI runs
it too.

## Versioning policy

Semantic versioning, against the token surface rather than the code:

| Change | Bump |
|---|---|
| A token is removed, or renamed | Major |
| A token's value changes enough to be a visible redesign | Major |
| A new token is added | Minor |
| A value is corrected without changing the design intent (a contrast fix, a rounding fix) | Patch |
| Documentation, build script internals, output formatting | Patch |

Consumers pin a range and upgrade deliberately. Because both the documentation site and the
showcase resolve their own dependency, one can upgrade ahead of the other; the version each is on
is visible in its lockfile, so a divergence is a fact someone can look up rather than something
that has to be noticed by eye.

Every release is recorded in [CHANGELOG.md](./CHANGELOG.md).
