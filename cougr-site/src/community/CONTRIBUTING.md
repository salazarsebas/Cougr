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

Run the relevant checks before opening a pull request:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

If you modify an example project, also run that example's local checks from its own directory. If the example supports Soroban contract builds, validate that flow as well.

## Documentation Expectations

Documentation should be professional, current, and proportionate:

- avoid stale exact counts when the repository is expected to grow
- explain decisions and usage patterns without turning every page into a long-form essay
- use tables when they improve scanability, not as a default for all content
- keep root-level documentation limited to material with clear long-term value

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
