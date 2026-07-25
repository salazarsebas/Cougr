# cougr-cli

The Cougr command-line tool. Two commands ship today:

- `cougr doctor` — local toolchain diagnostics. Checks Rust toolchain, `wasm32v1-none`
  target, `cargo`, and the `stellar` CLI; reports pass/fail per requirement with an
  actionable fix command for every failure. Exits non-zero if any check fails.

- `cougr new <name>` — scaffold a new Soroban contract crate wired to `cougr-core`.
  The first time `cougr new` runs on a machine, it invokes `cougr doctor` non-fatal
  (warnings to stderr) so environment problems are surfaced before they cause a
  confusing downstream error. Pass `--no-doctor` to skip.

## Install

```
cargo install cougr-cli
```

## Usage

```
$ cougr doctor
[1/4] rust toolchain            PASS  rustc 1.85.0 (meets 1.70.0)
[2/4] wasm32v1-none target      FAIL  not installed
                                  -> rustup target add wasm32v1-none
[3/4] cargo                     PASS  cargo 1.85.0
[4/4] stellar CLI               PASS  stellar 23.1.0 (meets 21.0.0)

3/4 checks passed.
error: toolchain checks failed; see messages above.
$ echo $?
1
```

## Stability

`cougr-cli` is published alongside `cougr-core` and follows the same release cadence.
Both ship at version `1.1.0` and bumping is done in lockstep with `Cargo.lock` updates
that touch either crate.

## License

MIT — see [`LICENSE`](../../LICENSE) in the repository root.
