## What changed

<!-- Describe the change concisely. Reference the issue this resolves if one exists. -->

Closes #

## Why

<!-- Explain the motivation. What problem does this solve or what gap does it fill? -->

## How it was validated

<!-- List the checks you ran. The baseline is the three commands from CONTRIBUTING.md: -->

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] Affected example(s) build and their tests pass

## Follow-up work or constraints

<!-- Anything reviewers should know: known limitations, deferred work, migration notes. Delete this section if there is nothing to flag. -->

---

## Public API checklist

*Complete this section only if the PR adds, removes, or changes public Rust symbols. Delete it otherwise.*

- [ ] The symbol belongs to the curated onboarding path or an intentional namespace (`accounts`, `zk::stable`, `zk::experimental`, etc.)
- [ ] Stable, beta, experimental, and test-only surfaces are not mixed in the same default entrypoint
- [ ] New public names do not duplicate an existing public concept
- [ ] Root-level re-exports are intentional and minimal
- [ ] Examples and integration tests use the sanctioned public path, not deep internal module paths
- [ ] Documentation is updated to match the actual exported API
- [ ] `CHANGELOG.md` entry added under the appropriate heading
