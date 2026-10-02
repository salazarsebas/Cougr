# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, based on the canonical
[`{{source_example}}`](https://github.com/salazarsebas/Cougr/tree/main/examples/{{source_example}})
example.

## Purpose and pattern

A match with two entities each holding a numeric field. `StorageWorld` (the
`incremental` template) persists only dirty entities on `flush()`, so an
unchanged entity is not rewritten - reducing gas costs. This is the key
difference from `SimpleWorld`, which writes the whole world in one instance
entry.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `spawn_player1` | - | `u32` | Create the first player entity and return its ID |
| `spawn_player2` | - | `u32` | Create the second player entity and return its ID |
| `set_score1` | `entity_id: u32`, `score: u32` | - | Set player 1's score, only that entity is rewritten |
| `set_score2` | `entity_id: u32`, `score: u32` | - | Set player 2's score, only that entity is rewritten |
| `score1` | `entity_id: u32` | `Option<u32>` | Player 1's current score, or `None` if unspawned |
| `score2` | `entity_id: u32` | `Option<u32>` | Player 2's current score, or `None` if unspawned |
| `entity_count` | - | `u32` | Number of entities spawned so far |

## Architecture overview

```
lib.rs         contract entrypoints - load world, call a rule, save world
  ├─ components.rs   Score1, Score2 (plain), player constants
  └─ systems.rs      validation rules, no storage access
```

Each entrypoint follows the same three beats: `load_world`, apply a pure rule
from `systems.rs`, `save_world`. Because the rules never touch storage they can
be tested directly, and the contract functions stay short enough to audit.

## Storage model

The `incremental` template uses **`StorageWorld`** with dirty-tracking. The
entire world is not written on every call - only entities whose components
changed since the last `flush()` are persisted. This means:

- **Touched entity**: its components are written to Soroban `persistent` storage
- **Untouched entity**: no storage write occurs, preserving gas and avoiding
  unnecessary key writes

The `"world"` instance-storage key wires up `impl_soroban_game!`. Components
live in per-entity persistent storage under `"cougr_ent"` + entity ID keys,
with component data under `"cougr_cmp"` + entity ID + component type keys.

## Main gameplay flow

1. A client calls `spawn_player1` and stores the returned entity ID.
2. A client calls `spawn_player2` and stores the returned entity ID.
3. The client calls `set_score1` with a new score; only player 1's entity is
   rewritten on flush, player 2's entity is left alone.
4. The client calls `set_score2`; only player 2's entity is rewritten.
5. `score1` and `score2` read state back without a write.

## Cougr APIs used

| API | Why |
| --- | --- |
| `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `StorageWorld` | Incremental per-entity persistent storage with dirty tracking |
| `test::GameHarness`, `Scenario`, `WorldFixture` | Sandbox for full-round integration tests |
| `cougr check` | Validates world consistency and storage invariants |

## Build and test

```bash
cargo test
stellar contract build
```

`stellar contract build` needs the WASM target and the Stellar CLI:

```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
```

The build writes `target/wasm32v1-none/release/{{module_name}}.wasm`, which is
what you deploy with `stellar contract deploy`.

## Known limitations

* No authorization: any caller can set any entity's score. Add `require_auth`
  and an owner component before putting this on a public network.
* Only two entities are supported - the template demonstrates partial updates,
  not a general-purpose ECS.
* Score values are unbounded - watch for u32 overflow in production.