# Repository Strategy

*Part 11 of 17. Monorepo vs. multi-repo, with a specific, justified recommendation rather than a menu of options.*

## Recommendation: stay a single repository for everything Rust, split out anything with a genuinely different toolchain

Keep `salazarsebas/Cougr` as the home for `cougr-core`, its internal subcrates, all 39 (and future) examples, and a new `cougr-cli` crate added as a workspace member. Split into new, separate repositories only for the documentation/showcase site and the TypeScript client SDK, and only when those are actually built, not preemptively.

## Why the current core stays together

The examples and the core crate are tightly, usefully coupled: `EXAMPLE_STANDARD.md` compliance depends on the exact version of `cougr-core` in the same repository, CI already tests examples against the in-tree crate via path dependencies for exactly this reason (per the audit in [01-current-state-assessment.md](./01-current-state-assessment.md)), and a single PR that changes a public API can and should update the examples that use it in the same change set. Splitting examples into a separate repository would immediately reintroduce the cross-repo version-skew problem the project is already fighting to eliminate (the 7 canonical examples on local path dependencies, pending a publish catch-up). This is the Turborepo/Bevy lesson from [02-market-research.md](./02-market-research.md): a monorepo earns its keep specifically when coordinated releases across many packages are valuable, and Cougr's core-plus-examples relationship is exactly that case.

Adding `cougr-cli` to the same workspace, rather than a separate repository, follows the same logic: the CLI's `cougr new`/`cougr add` commands read directly from the example catalog and `EXAMPLE_STANDARD.md` conventions living in this repository. A separate CLI repository would need to either vendor or fetch that catalog data at build or runtime, adding a synchronization problem for no benefit, since both crates already share one CI, one Rust toolchain, and one release cadence.

## Why the documentation/showcase site should be a separate repository

The moment the docs site and showcase gallery move from "plain Markdown in `docs/`" to an actual generated site (mdBook or a static site generator, per [12-documentation-architecture.md](./12-documentation-architecture.md)), it introduces a different build toolchain, a different deploy target (GitHub Pages, on its own release cadence independent of crate versions), and a different desirable contributor permission boundary: a technical writer or designer should be able to fix a typo or update the showcase gallery without needing crates.io publish rights or triggering the Rust CI's `clippy -D warnings` gate on unrelated Markdown changes. A separate `cougr-site` repository (or `cougr-docs`) cleanly isolates this. It should still consume the core repository's content as its source of truth (via a documented sync step pulling `docs/*.md` and the example catalog metadata at build time, not by duplicating content), preserving one authoritative source while allowing an independent release cadence and contributor base.

## Why the client SDK should be a separate repository

Once the TypeScript client SDK in [06-product-strategy.md](./06-product-strategy.md) is built, it belongs in its own repository (`cougr-sdk-js` or similar) for the same toolchain-divergence reason: an npm-published package has its own versioning scheme (semver against the JS ecosystem's expectations, not lockstep with `cougr-core`'s Rust version), its own CI (Node, not Rust/WASM), and its own review needs (frontend/TypeScript reviewers, not necessarily the same people reviewing Soroban contract code). Bundling it into the main repository would mean every Rust contributor's CI runs Node tooling they don't need, and vice versa.

## The general rule for future splits

Split into a new repository when a component's toolchain, release cadence, or intended contributor base diverges enough from the core Rust workspace that keeping it co-located adds CI noise or permission friction without adding coordination benefit. Do not split based on component "importance" or team excitement, several Tier 2/3 components in [05-ecosystem-vision.md](./05-ecosystem-vision.md), like the Skills catalog, are Rust/Markdown-native and should stay in the main repository even though they are conceptually separate products, because they share the toolchain and benefit from being versioned alongside the CLI they wrap.

## What this means practically, near-term

No repository split is needed today. The immediate action is additive: bring `cougr-cli` into the existing workspace as a new member. The first repository split (`cougr-site`) should happen when the documentation site work in [12-documentation-architecture.md](./12-documentation-architecture.md) begins, not before, since there is nothing to migrate out of the main repository until that content generation work exists.
