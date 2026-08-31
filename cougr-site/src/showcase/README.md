# Showcase

The Showcase is a live directory of games and demos built with Cougr. It is
generated automatically from the [example catalog](https://github.com/salazarsebas/Cougr/blob/main/examples/catalog.toml)
and each example's own `README.md` — no manual duplication.

## Browsing the gallery

- **[Example Gallery](gallery.md)** — filter by category and maturity to find
  the right reference for your use case.
- Click any card to see the full detail page, pulled directly from that
  example's `README.md`.

## How it works

The gallery is a **static, build-time-generated** set of pages — zero backend,
zero database. The generator (`cougr-site/generate-showcase.py`) reads:

1. `examples/catalog.toml` — structured metadata (category, maturity, Cougr
   features, optional screenshot/testnet contract).
2. Each example's `README.md` — the description and full documentation.
3. `packages/tokens/tokens.json` — design tokens (colors, typography, spacing)
   consumed as CSS custom properties, so the showcase never visually diverges
   from the docs site.

To generate locally:

```bash
python3 cougr-site/generate-showcase.py
mdbook serve cougr-site
```

## Preview images

Examples with a `preview.svg` in their directory display it in the gallery
card and detail page. Examples without a preview image render cleanly without
any broken image tags.

## Submit your game

Once your example is cataloged in `examples/catalog.toml` and meets the
[quality standard](https://github.com/salazarsebas/Cougr/blob/main/examples/EXAMPLE_STANDARD.md),
it will appear in the gallery automatically. A **"Cougr Verified"** badge can
be earned by opening a PR — the catalog's `verified` field controls whether the
badge renders.
