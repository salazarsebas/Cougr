# {{crate_name}}

{{description}}

Generated with `cougr new --template {{template_id}}`, based on the canonical
[`{{source_example}}`](https://github.com/salazarsebas/Cougr/tree/main/examples/{{source_example}})
example.

## Purpose and pattern

A player enters the world with `spawn` and walks around it one step at a time
with `move_entity`. It is the smallest complete Cougr game: typed ECS components
stored in a `SimpleWorld`, persisted through `SorobanGame`, with position changes
published as indexed Soroban events for off-chain clients.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `spawn` | — | `u32` | Create a new entity at the origin and return its ID |
| `move_entity` | `entity_id: u32`, `direction: u32` | — | Move one step; `0` N, `1` E, `2` S, `3` W |
| `position` | `entity_id: u32` | `Option<Position>` | Current position, or `None` if unspawned |
| `moves` | `entity_id: u32` | `Option<Moves>` | Remaining move budget and last direction |
| `entity_count` | — | `u32` | Number of entities spawned so far |

## Architecture overview

```
lib.rs         contract entrypoints — load world, call a rule, save world
  ├─ components.rs   Position (observed), Moves (plain), direction constants
  └─ systems.rs      step() — pure movement rule, no storage access
```

Each entrypoint follows the same three beats: `load_world`, apply a pure rule
from `systems.rs`, `save_world`. Because the rules never touch storage they can
be tested directly, and the contract functions stay short enough to audit.

## Storage model

The entire `SimpleWorld` lives in **instance storage** under the `"world"` key,
wired up by `impl_soroban_game!({{ContractName}}, "world")`. One entity's
`Position` and `Moves` are two component tables inside that world, so a whole
game state is a single read and a single write per call.

## Main gameplay flow

1. A client calls `spawn` and stores the returned entity ID.
2. The entity is placed at `(0, 0)` with a budget of 1,000 moves, and a
   `(COUGR, set, position)` event is emitted.
3. The client calls `move_entity` with a direction; the position shifts by one
   and the budget drops by one, emitting another position event.
4. `position`, `moves`, and `entity_count` read state back without a write.

## Cougr APIs used

| API | Why |
| --- | --- |
| `impl_component_observed!` | `Position` changes must reach indexers without polling |
| `impl_component!` | `Moves` is read on demand, so it does not need events |
| `SorobanGame` / `impl_soroban_game!` | Removes hand-written world load/save boilerplate |
| `SimpleWorld` | Typed component storage for a growing entity population |
| `test::GameHarness`, `Scenario`, `WorldFixture` | Sandbox for full-round integration tests (see [Testing Guide](../../docs/learn/TESTING_GUIDE.md)) |

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

* No authorization: any caller can move any entity. Add `require_auth` and an
  owner component before putting this on a public network.
* The world is unbounded — entities can walk arbitrarily far from the origin.
* The move budget is fixed at spawn and cannot be topped up.
