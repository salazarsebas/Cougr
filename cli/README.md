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
| `incremental` | `spawn_and_move` | Two players set scores with `StorageWorld` dirty-tracking; only changed entities are rewritten on `flush` |

Templates are embedded in the binary at compile time, so `cougr new` works
offline.

| Template | Based on | What you get |
| --- | --- | --- |
| `starter` | `examples/spawn_and_move` | Spawn entities and move them around a 2D world, with observed components emitting indexed events |
| `turn-based` | `examples/tic_tac_toe` | Two players alternating on a board, with rich `Address`/`Vec` components |
| `hidden-info` | `examples/hidden_hand` | Hidden hands verified with Groth16 proofs via `circuits::hidden_cards` |
| `session-auth` | `examples/session_arena` | Approve a session once, play without wallet prompts, fall back to owner auth on expiry |
| `incremental` | `spawn_and_move` | Two players set scores with `StorageWorld` dirty-tracking; only changed entities are rewritten on `flush` |

### When to stay on `SimpleWorld` instead

The `SimpleWorld` template is appropriate when:

- Your game state fits in a single instance-storage write per call - the overhead of
  per-entity persistent storage isn't justified.
- You have many entities that all change together frequently - `SimpleWorld` writes
  the whole world in one operation, which can be more efficient than multiple partial
  writes.
- You need rich query capabilities ( `get_entities_with_component`, `table_index`,
  `all_index`) that `StorageWorld` doesn't support natively.
- Your components are mostly observed (emit indexer events) rather than read back
  through contract calls.

The `incremental` template (`StorageWorld`) is appropriate when:

- You have a small number of entities (2-10) and only a subset change per turn.
- Gas cost matters - partial writes to persistent storage are cheaper than writing
  the entire world state each call.
- You want to prove that untouched entities are NOT rewritten on `flush()`.
- You can accept the overhead of per-entity keys and dirty-tracking metadata.

## `cougr export`

```bash
cougr export my-game --config turn-based.json --path ./projects
cd projects/my-game
cougr check
cargo test
stellar contract build
```

`turn-based.json` uses the studio `TurnBasedConfig` shape:

```json
{"board_width": 3, "board_height": 3, "win_length": 3}
```

Width and height must be 3 through 8; win length must be 3 through the
smaller dimension. Invalid configurations fail before the project directory is
created. Export uses the same embedded `turn-based` template, rendering and
writer as `cougr new`, and refuses to overwrite an existing directory. The
generated README and CLI output retain the same build and test next steps.

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
