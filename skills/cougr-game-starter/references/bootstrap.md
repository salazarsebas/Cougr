# Project Bootstrap

Use this reference after scaffolding a project with `cougr new` (see [SKILL.md](../SKILL.md) Step 2). It covers what the CLI does not: extending the generated layout, and dependency strategy for the one case `cougr new` does not handle, local development against an unpublished `cougr-core`.

## Generated Shape

`cougr new <name> --template <template>` produces a single crate:

```text
my-game/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── components.rs
│   ├── systems.rs
│   └── test.rs
```

Collapse files further only if the template already over-splits a genuinely tiny prototype. Split beyond this shape when game logic becomes harder to scan than to navigate, not before.

## Dependency Strategy

`cougr new` pins the generated project to the published `cougr-core` release from crates.io. Only override this when the user is working locally against an unreleased `cougr-core`, for example while developing a feature in this repository alongside a generated example:

```toml
[dependencies]
soroban-sdk = "25.1.0"
cougr-core = { path = "../path/to/cougr-core" }
```

Revert to the published version before the project is meant to build standalone.

## Build Target

Use `wasm32v1-none` as the WASM target for Soroban-oriented builds.

Common commands:

```bash
rustup target add wasm32v1-none
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --target wasm32v1-none --release
```

If `stellar contract build` is available in the project workflow, use it where appropriate.

## Starter Module Responsibilities

| File | Responsibility |
|---|---|
| `src/lib.rs` | Contract entrypoints and top-level wiring |
| `src/components.rs` | Game-facing component definitions |
| `src/systems.rs` | State transitions and gameplay logic |
| `src/test.rs` | Match-flow and rules tests |

State shape lives wherever the chosen template puts it (usually alongside components in `lib.rs` or `components.rs`); do not add a separate `state.rs` unless the game's state genuinely needs its own module.

## Starter Design Rules

- Keep the initializer simple and deterministic.
- Keep action methods narrow.
- Return small, useful state snapshots to help tests and clients.
- Avoid building a generalized engine wrapper before the game loop exists.
- Add new files only when they reduce cognitive load.
- Extend the generated skeleton in place. Do not regenerate a hand-written contract skeleton alongside it.
