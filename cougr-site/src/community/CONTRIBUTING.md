# Contributing

Contributions should improve the framework, the example catalog, or the supporting documentation with a clear purpose. This repository is structured to be useful both as a reusable library and as a reference codebase, so changes should optimize for correctness, clarity, and maintainability.

## Scope

Good contributions typically fall into one of these categories:

| Area | Expected outcome |
|---|---|
| Core framework | Improved ECS, scheduling, storage, authorization, or zero-knowledge capabilities |
| Examples | New game patterns, better reference implementations, or tighter example documentation |
| Documentation | Clearer architecture, setup, or usage guidance aligned with the current codebase |
| Quality | Better tests, tooling, validation, or CI coverage |

## Development Standards

- Keep changes focused. Avoid mixing unrelated refactors with feature work.
- Update documentation when behavior, structure, or public APIs change.
- Prefer clear names and straightforward control flow over clever abstractions.
- Preserve repository consistency. New files should fit the existing layout and conventions.
- Do not add generated reports, ad hoc summaries, or temporary planning documents to the repository root.

## Local Validation

Run this from the repository root before you open or update a pull request:

```bash
make check
```

It runs the same checks as CI for the parts of the repository your branch changes (core library, CLI, examples, docs site, catalog, SDKs, clients, studio) and ends with a pass/fail summary. Fix everything it reports and run it again until it passes. A pull request is only merged when every CI workflow passes.

- `make fix` applies the automatic fixes: `cargo fmt` and the docs site sync.
- `make check-all` runs every check, whatever you changed.
- If a tool is missing (for example `stellar`, `mdbook`, or `cargo-deny`), `make check` tells you how to install it. Skipped checks still run in CI.
- CI tests your branch merged into `main`. Keep it current with `git fetch upstream && git rebase upstream/main` (add the remote once with `git remote add upstream https://github.com/salazarsebas/Cougr.git`).

If you use an AI assistant, see [AGENTS.md](https://github.com/salazarsebas/Cougr/blob/main/AGENTS.md).

## Documentation Expectations

Documentation should be professional, current, and proportionate:

- avoid stale exact counts when the repository is expected to grow
- explain decisions and usage patterns without turning every page into a long-form essay
- use tables when they improve scanability, not as a default for all content
- keep root-level documentation limited to material with clear long-term value
- follow the terminology and voice rules in [docs/VOICE_GUIDE.md](docs/VOICE_GUIDE.md) for all doc, example, and marketing copy

## Pull Requests

Pull requests should make it easy to review technical intent. A strong PR description usually covers:

1. what changed
2. why the change was needed
3. how it was validated
4. any follow-up work or constraints reviewers should know about

## Adding Examples

When adding a new example:

- make the example self-contained
- include a local `README.md`
- keep the example focused on one or two clear patterns
- add CI coverage when the example is meant to remain a maintained reference
- add a matching `[example.<name>]` entry to `examples/catalog.toml` and validate the catalog locally (requires Python 3.11 or newer):

  ```bash
  python3 scripts/validate_catalog.py
  ```

  The validator checks required fields, allowed `category` and `maturity` values, that each `name` maps to a directory under `examples/`, that `cougr_features` is a non-empty list, and that any `screenshot` path exists. `examples/catalog_broken.toml` is a deliberately invalid fixture; running the validator against it must exit non-zero, which is how CI confirms the rules still fire:

  ```bash
  python3 scripts/validate_catalog.py examples/catalog_broken.toml  # expected to fail
  ```

## Review Criteria

Changes are more likely to be accepted when they:

- solve a real problem in the framework or examples
- keep the API and repository structure coherent
- include appropriate validation
- improve the repository without increasing maintenance noise

## Public API Checklist

Changes that touch public Rust APIs should be reviewed against this checklist before merge:

- the symbol belongs to the curated onboarding path or an intentional namespace such as `accounts`, `zk::stable`, or `zk::experimental`
- stable, beta, experimental, and test-only surfaces are not mixed in the same default entrypoint
- new public names do not duplicate an existing public concept
- root-level re-exports are intentional and minimal
- examples and integration tests use the sanctioned public path instead of deep internal module paths
- documentation is updated to match the actual exported API

## Code of Conduct

All contributors are expected to follow the project's [Code of Conduct](CODE_OF_CONDUCT.md). Reports of unacceptable behavior can be sent to the project maintainers through GitHub.
