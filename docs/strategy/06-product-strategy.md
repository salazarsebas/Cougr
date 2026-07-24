# Product Strategy

*Part 7 of 17. Concrete product specification for the Tier 1 and Tier 2 components named in [05-ecosystem-vision.md](./05-ecosystem-vision.md). This is a specification of scope and behavior, not implementation code, per the research constraints.*

## The CLI: `cougr`

This is the single highest-priority product decision in the entire package. It should ship as a standalone Rust binary, installable via `cargo install cougr-cli` (a genuinely separate, published crate; see [10-repository-strategy.md](./10-repository-strategy.md) for why this stays in the same repository as `cougr-core`), with a small, memorable command surface rather than an exhaustive one.

- **`cougr new <name> [--template <name>]`**: scaffolds a new Soroban contract crate with `cougr-core` wired up, a working `lib.rs` following the canonical `lib.rs`/`components.rs`/`systems.rs` layout already defined by `EXAMPLE_STANDARD.md`, a passing test using `GameHarness`, and correct `Cargo.toml` metadata. The template flag selects a starting point from a small, curated set (`starter`, `turn-based`, `hidden-info`, `session-auth`), each backed by one of the existing canonical examples rather than a new, separately maintained template set, this is the shadcn-style "distribute existing good source" pattern from [02-market-research.md](./02-market-research.md), not new content to maintain.
- **`cougr add <piece>`**: pulls a specific component/system pair (for example `session-auth`, `hidden-hand`, `standards/pausable`) into an existing project, copied as owned source the developer can modify, sourced directly from the example library and `ops`/`accounts` modules. This directly operationalizes the `EXAMPLE_STANDARD.md` catalog that already exists but currently has no distribution mechanism beyond manual copy-paste.
- **`cougr check`**: runs the same hygiene and standard-compliance checks `scripts/enforce_hygiene.sh` already performs internally, exposed as a first-class command a contributor or external developer can run locally before opening a PR. This turns existing internal tooling into a public product surface at near-zero additional engineering cost.
- **`cougr doctor`**: verifies the local toolchain (Rust version, `wasm32v1-none` target, `stellar` CLI presence and version) and gives actionable fix instructions, addressing the "works on my machine" class of onboarding failure before it happens.

Explicitly out of scope for v1: a `deploy` subcommand duplicating `stellar contract deploy`. Wrapping an already-good tool adds a maintenance burden and a second source of truth for no real gain; `cougr new`'s scaffolded README should simply document the two or three `stellar` CLI commands needed next, keeping Cougr's CLI focused on what only Cougr can provide (scaffolding tied to its own conventions).

## The client SDK (TypeScript)

A focused package, not a general Stellar SDK wrapper (that already exists as `stellar-sdk` / Stellar Wallets Kit and should be depended on, not duplicated). Scope: (1) a typed client for constructing and submitting transactions against `cougr-core`'s session/account primitives, mirroring the Rust-side `SessionBuilder` API so the mental model transfers across languages, (2) a passkey/WebAuthn registration and sign-in flow that pairs with `Secp256r1Storage`, and (3) an event-subscription helper that decodes the `(COUGR, set, <type>)` observed-component events into typed objects. This is deliberately the same shape as the gap identified against Cartridge/MUD in [03-competitive-analysis.md](./03-competitive-analysis.md), sized to what Cougr can credibly maintain (three focused capabilities) rather than an attempt to out-build either.

## The showcase / example gallery

A static, generated page (data-driven off the existing `examples/README.md` catalog and each example's own `README.md` frontmatter, extended with a `category`, `maturity`, and optional `screenshot`/`testnet_contract` field) rendered as a browsable gallery. No backend, no database, this is a build-time generation step so it can be hosted on GitHub Pages alongside the documentation site with zero incremental infrastructure. Each entry should answer, in under ten seconds of reading: what game is this, what Cougr feature does it demonstrate (privacy, session auth, standards, arcade real-time), and how mature is it. This is the single highest-ROI content project available, because the content (39 working games) already exists; only the presentation layer is missing.

## Standards layer positioning

`ops` (the `Ownable`/`AccessControl`/`Pausable`/execution-guard module) is currently documented as one module among several. Product-wise it deserves separate framing as "the OpenZeppelin of Soroban gaming," because that framing is instantly legible to any developer who has touched EVM tooling and immediately communicates both maturity and intent. This is a documentation and messaging change, not a code change, and should be reflected in the README's value-prop section and the docs site's information architecture (see [12-documentation-architecture.md](./12-documentation-architecture.md)).

## What ships together vs. independently

Consistent with the composability principle: the CLI, client SDK, showcase, and Skills catalog are each independently useful and independently versioned (the CLI does not require the SDK, the showcase does not require the CLI), but they are designed so that using one naturally surfaces the next, `cougr new`'s scaffolded README links to the docs site tutorial that matches the chosen template, which links to the showcase entry for the canonical example it was based on, which credits the CLI as the fastest way to start your own. This is the connective tissue described in [05-ecosystem-vision.md](./05-ecosystem-vision.md), implemented as cross-links and shared data, not shared code.
