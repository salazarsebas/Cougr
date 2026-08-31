# cougr-cli

Command-line tooling for [Cougr](https://github.com/salazarsebas/Cougr), the ECS
framework for on-chain games on Stellar/Soroban.

```bash
cargo install cougr-cli

cougr new my-game --template starter
cd my-game
cargo test
```

## `cougr new`

```
cougr new <NAME> [--template <TEMPLATE>] [--path <DIR>]
```

Scaffolds a Soroban contract crate wired to `cougr-core`, following the
`lib.rs` / `components.rs` / `systems.rs` layout defined by
[`EXAMPLE_STANDARD.md`](../examples/EXAMPLE_STANDARD.md), with a passing
`GameHarness` test suite. The generated crate depends on the published
`cougr-core` release, so it builds outside this repository.

| Flag | Default | Description |
| --- | --- | --- |
| `--template`, `-t` | `starter` | Which starting point to generate |
| `--path` | current directory | Where to create the project directory |

### Templates

Each template is derived from a canonical example, so the code you get is the
same code the framework's own reference projects run.

| Template | Based on | What you get |
| --- | --- | --- |
| `starter` | `examples/spawn_and_move` | Spawn entities and move them around a 2D world, with observed components emitting indexed events |
| `turn-based` | `examples/tic_tac_toe` | Two players alternating on a board, with rich `Address`/`Vec` components |
| `hidden-info` | `examples/hidden_hand` | Hidden hands verified with Groth16 proofs via `circuits::hidden_cards` |
| `session-auth` | `examples/session_arena` | Approve a session once, play without wallet prompts, fall back to owner auth on expiry |

Templates are embedded in the binary at compile time, so `cougr new` works
offline.

## `cougr add`

```bash
cougr add --list
cougr add session-auth
cougr add hidden-hand
cougr add standards/pausable
```

Adds an embedded capability as editable source to the current project and
updates `src/lib.rs` automatically. Piece files are never overwritten: a
second add reports the files that would have been written. Pieces and their
descriptions are defined in [`pieces/pieces.toml`](pieces/pieces.toml) and are
embedded in the CLI binary for offline use.

## Development

```bash
cargo test -p cougr-cli                 # unit and CLI tests
cargo test -p cougr-cli -- --ignored    # generate all 4 templates and cargo test each
```

Template sources live in [`templates/`](templates/). Files there use two naming
conventions that the CLI undoes when writing a project:

* `Cargo.toml.tmpl` → `Cargo.toml`. Cargo skips subdirectories containing a
  manifest when packaging a crate, which would drop the template manifests from
  the published binary.
* `gitignore` → `.gitignore`, so the file ships as content instead of applying
  to this repository.

Inside a template, `{{crate_name}}`, `{{module_name}}`, `{{ContractName}}`,
`{{description}}`, `{{template_id}}`, `{{source_example}}`,
`{{cougr_core_version}}`, and `{{soroban_sdk_version}}` are substituted at
generation time. `cargo test -p cougr-cli` fails if a placeholder survives, if a
template drops a file from the canonical layout, if a manifest reaches for a
path dependency, or if a test module stops using `GameHarness`.
