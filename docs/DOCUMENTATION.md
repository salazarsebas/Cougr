# Documentation

## Purpose

This document records the documentation quality bar for `cougr-core` after the
`1.0.0` crates.io publication.

## Installation Source

Released documentation should point users to the published crate:

```toml
[dependencies]
cougr-core = "1.0.0"
```

Do not present repository dependencies as the default installation path in
release-facing docs.

## Rustdoc Checks

Before publishing documentation changes, run:

```bash
RUSTDOCFLAGS="--cfg docsrs" cargo doc --no-deps --all-features
```

The build should finish without broken intra-doc link warnings.

## Missing Docs Strategy

Cougr has a broad public API, so missing-doc enforcement should be introduced
gradually instead of enabling a crate-wide hard gate all at once.

Priority order:

1. Stable onboarding facades: `app`, `prelude`, root re-exports
2. Stable ECS modules: `simple_world`, `query`, `scheduler`, `component`
3. Stable standards: `ops` / `standards`
4. Stable privacy surface: `privacy::stable` / `zk::stable`
5. Beta and Experimental modules after their contracts settle

New stable public items should include rustdoc that states what the item is for,
which maturity tier it belongs to, and any storage or security assumptions that
callers need to preserve.

## docs.rs Links

Do not add a docs.rs badge, manifest `documentation` URL, or versioned docs.rs
link until that URL has been opened and verified for the published version.

When a verified page exists, prefer a stable crate page first and use a
versioned URL only when the version-specific page is known to exist.
