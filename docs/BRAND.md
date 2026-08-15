# Cougr Brand

Part of the design system called for in [docs/strategy/09-design-strategy.md](./strategy/09-design-strategy.md)
(design tokens + logo system) and closes out
[#259](https://github.com/salazarsebas/Cougr/issues/259). Scope, per that strategy doc: a small,
fixed vocabulary — not a large palette or a decorative visual language. Cougr reads as
infrastructure, closer to Linear or Vercel's register than a game or a "blockchain-cold" brand.

This document is the single source of truth for color, type, spacing, and the logo system. Anything
consuming these values (docs site, showcase, README, CLI output styling) should point back here
rather than re-deriving values independently.

## Color palette

Every pair below is the color used in that mode; each has been checked against its *own* mode's
background, not against the other mode's. All pairs meet WCAG AA (4.5:1), including the tier colors,
which is stricter than the 3:1 large-text/UI-component minimum the maturity tags actually need —
headroom intentionally left so these colors stay usable for body text too, not just badges.

### Neutrals

| Token | Light mode | Dark mode |
|---|---|---|
| `color-bg` | `#FFFFFF` | `#14100D` |
| `color-surface` (cards, code blocks) | `#F6F4F1` | `#1F1A15` |
| `color-text` | `#1A1512` | `#F3EDE4` |
| `color-text-secondary` | `#5B534B` | `#B7ABA0` |

Contrast on own background: light text 18.10:1, light text-secondary 7.54:1, dark text 16.26:1,
dark text-secondary 8.42:1.

### Primary and accent

One primary (the logo's coat tone, functioning as the brand hue) and one accent (interactive
elements — links, focus states — kept distinct from the brand hue so "this is brand" and "this is
clickable" never get confused).

| Token | Light mode | Dark mode | Contrast on own bg |
|---|---|---|---|
| `color-primary` | `#8A5A22` | `#D9A15C` | 5.89:1 / 8.29:1 |
| `color-accent` | `#2E5F8A` | `#6FA8D9` | 6.73:1 / 7.46:1 |

### Maturity tiers (Stable / Beta / Experimental)

Makes the [MATURITY_MODEL.md](./MATURITY_MODEL.md) taxonomy — already the project's strongest piece
of design discipline — visible everywhere it's referenced: README, CHANGELOG, SECURITY.md, docs
site, showcase, rustdoc, and CLI scaffolding output. Same three colors, same meaning, every time.

| Tier | Light mode | Dark mode | Contrast on own bg |
|---|---|---|---|
| Stable | `#1C7A4D` | `#4FBE8A` | 5.33:1 / 8.17:1 |
| Beta | `#9A6B00` | `#E3A72E` | 4.69:1 / 8.86:1 |
| Experimental | `#8034B8` | `#C08DE8` | 6.77:1 / 7.40:1 |

Green/amber/purple was chosen over green/amber/red because red is already claimed by error/danger
states in most UI conventions; reusing it for "Experimental" (which is a normal, allowed state, not
an error) would misrepresent it as something broken.

## Typography

One interface/documentation typeface, one monospace, both system-first — consistent with the CSP
constraints called out in the design strategy for future docs-site/showcase work, and avoiding a
webfont loading cost for a docs-heavy product where text legibility matters more than a bespoke
typeface.

```css
--font-sans: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
             "Helvetica Neue", Arial, sans-serif;
--font-mono: ui-monospace, "JetBrains Mono", "Fira Code", "Cascadia Code",
             Consolas, "SF Mono", Menlo, monospace;
```

`--font-sans` is used for interface and documentation text (headings, body copy, showcase UI).
`--font-mono` is used identically for code blocks, the docs site, the showcase, and any
terminal-adjacent CLI output styling — one monospace choice, not a different one per surface.

## Spacing and radius scale

A small fixed set, not an open-ended range — the same reasoning as the color palette: a large scale
is a consistency risk, not a flexibility win.

| Token | Value |
|---|---|
| `space-1` | 4px |
| `space-2` | 8px |
| `space-3` | 12px |
| `space-4` | 16px |
| `space-5` | 24px |
| `space-6` | 32px |
| `space-7` | 48px |
| `space-8` | 64px |

| Token | Value | Use |
|---|---|---|
| `radius-sm` | 4px | inline elements, tags |
| `radius-md` | 8px | cards, inputs, buttons |
| `radius-lg` | 12px | panels, modals |
| `radius-full` | 9999px | pills, avatars, icon badges |

## Logo system

`public/Cougr.png` (the original raster mascot) is retained for backward compatibility, but
`public/Cougr.svg` is now the source-of-truth vector mark. It was produced by quantizing the
original artwork to this system's four logo tones and tracing each color region, rather than by
hand — so it reproduces the actual mascot (ear shape, muzzle, eye, coat shading) instead of a
freehand approximation.

**Logo tones** (fixed, used identically in light and dark contexts — the mark does not re-theme):

| Token | Hex | Role |
|---|---|---|
| `logo-ink` | `#171310` | outline, eye, nostril, mouth line, neck shadow |
| `logo-shadow` | `#6B4522` | mid-tone transition band |
| `logo-coat` | `#A06A2E` | top/back plane (ear, crown, neck, shoulder) |
| `logo-cream` | `#F1DCB8` | front-facing plane (forehead, cheek, muzzle, chin) |

The full-color mark was checked against both a white and a near-black (`#14100D`) background and
reads clearly on both — the ink outline recedes into a dark background rather than disappearing
against it, since the coat/cream/shadow tones carry the silhouette. No separate dark-mode
recolor of the icon itself was needed.

### Files and variants

| File | Variant | Use |
|---|---|---|
| `public/Cougr.svg` | icon-only, full color | favicon, avatar, any square slot |
| `public/brand/cougr-mono.svg` | icon-only, single color (`fill="currentColor"`) | watermarks, 1-color print, contexts that set their own color via CSS |
| `public/brand/cougr-lockup-light.svg` | icon + "COUGR" wordmark, ink text | README header, light site header |
| `public/brand/cougr-lockup-dark.svg` | icon + "COUGR" wordmark, cream text | dark site header |
| `public/brand/favicon-32.png`, `favicon-16.png` | rasterized icon-only | `<link rel="icon">` — SVG favicons aren't universally supported yet, so both are provided |
| `public/brand/apple-touch-icon-180.png` | rasterized icon-only | `<link rel="apple-touch-icon">` |
| `public/brand/cougr-icon-512.png` | rasterized icon-only | social/OG image, app icon source |

All SVGs are plain vector paths (no external font or asset references), so they render identically
anywhere without extra dependencies.

**Sizing note:** the mark holds up well down to ~32px. Below that (i.e. a 16px favicon tab icon)
fine detail — the eye and mouth line — reads as a soft blob rather than distinct features; this is
normal for a detailed mark at that size and matches how most favicons behave, but is worth knowing
if a future pass wants a further-simplified glyph specifically for 16px contexts.

## What's out of scope here

- Iconography (maturity tiers, game categories, doc callouts) and the voice/terminology glossary —
  both called out in the design strategy doc, but distinct enough in kind (icon set design; a
  written vocabulary list) to be their own follow-up rather than folded into this color/type/logo
  pass.
- Consuming these tokens as an actual code package (CSS variables / Tailwind config / etc.) —
  explicitly out of scope per #259, and now shipped separately as
  [`packages/tokens`](../packages/tokens). This document stays the source of truth; that package
  encodes it. Change a value here first, then mirror it there and rebuild.
- Any change to `README.md`'s current logo usage — a follow-up application of this system, not
  part of defining it.
