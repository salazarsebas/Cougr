# Design Strategy

*Part 10 of 17. Cougr currently has one logo and four shields.io badges as its entire visual identity. This defines what a premium, cohesive design system should specify, without prescribing pixel-level choices that belong to an actual design pass.*

## The design principle Cougr already has, and should build the visual system around

The Stable/Beta/Experimental maturity model, applied consistently across README, CHANGELOG, and SECURITY.md, is already the project's strongest piece of design discipline, it just has not been extended into visual form. The recommendation here is not to invent a new visual language from nothing, but to make the maturity model itself visible: a small, fixed set of colors and badge styles that mean the same thing everywhere a developer encounters them (docs site, showcase, crate-level rustdoc, CLI output). This is exactly the "small vocabulary, consistently applied" pattern observed across GitHub, Linear, and Figma in [02-market-research.md](./02-market-research.md), and it is close to free to build because the underlying taxonomy already exists and is already used correctly by the team.

## What the brand should communicate

Not playful or consumer-gaming (Cougr is infrastructure, not a game itself), and not generic blockchain-cold either. The honest-by-default documentation culture identified in [01-current-state-assessment.md](./01-current-state-assessment.md), stating gaps plainly rather than marketing around them, is a genuine differentiator worth making a visible brand trait, not just a documentation habit. A design system that reads as precise, confident, and unpretentious (closer to Linear or Vercel's register than to a flashy Web3 project's) matches both the actual engineering culture and the "premium quality over development speed" principle better than a louder, more game-flavored identity would.

## Design system specification (what should exist, not the literal values)

- **Color palette**: a small core palette (one primary, one or two accents, a neutral scale for text/background in both light and dark mode) plus the three maturity-tier colors (Stable/Beta/Experimental) used identically everywhere they appear. No more than this; a large palette is a maintenance burden and a consistency risk, not a strength.
- **Typography**: one typeface for interface/documentation text (a well-hinted, highly legible sans, matching the "read a lot of technical prose" reality of a docs-heavy product) and one monospace typeface for code, used identically in the docs site, the showcase, and any terminal-adjacent CLI output styling (colors/formatting the CLI itself prints).
- **Design tokens**: spacing scale, radius scale, and elevation/shadow rules defined once and consumed by both the documentation site and the showcase, so the two never visually diverge even though they may be separate static builds.
- **Iconography**: a small set of icons for the maturity tiers, the game categories used in the showcase (turn-based, real-time/arcade, hidden-information, puzzle), and common doc callouts (warning, note, security), kept to one consistent stroke weight and style.
- **Logo system**: the existing `Cougr.png` should be vectorized (SVG) and given a small set of approved lockups (icon-only for favicons/badges, icon-plus-wordmark for headers), replacing the single large PNG currently doing every job.
- **Voice and terminology glossary**: a short, enforced vocabulary list (for example: always "contract," never "smart contract" after first use; always "component"/"system" per ECS convention, never "entity data" or other ad hoc phrasing) so documentation, showcase copy, and marketing copy never drift into inconsistent terminology, a common failure mode called out explicitly in [02-market-research.md](./02-market-research.md).

## Where design work applies first

In order: the documentation site and showcase (the two new public surfaces being built in Tier 1), then the README and existing docs (a lighter pass, applying the same tokens without a rewrite), then the CLI's terminal output (color-coded maturity tags on scaffolded output, consistent with the web-facing system), then, only once all of the above exist, any conference/marketing collateral. Building marketing collateral before the docs site and showcase exist would be polishing something with no destination to send visitors to.

## What premium does not mean here

Premium does not mean heavy, animated, or visually loud. Given the primary audience is developers reading technical content for extended periods, the actual premium signal is restraint: fast page loads, high text legibility, minimal decorative motion, and consistent information density. The competitive analysis in [03-competitive-analysis.md](./03-competitive-analysis.md) already shows Cougr's engineering substance outpaces its current presentation; the fix is precision and consistency, not spectacle.
