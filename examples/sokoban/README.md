# Sokoban

A push-crate puzzle contract demonstrating `ArchetypeWorld`, Cougr's archetype-grouped ECS backend, and `ArchetypeQueryBuilder`'s `without_component` query for win detection.

## Purpose and pattern

The player moves one tile at a time; pushing into a crate shoves it ahead if the tile beyond is free. The level is solved once every crate shares a position with a goal.

Every other canonical example builds on `SimpleWorld`. Sokoban is the canonical reference for `ArchetypeWorld` instead: its five component types (`Player`, `Wall`, `Crate`, `Goal`, `OnGoal`, plus the shared `Position`) each define a distinct archetype by construction, so the win check and push resolution query a pre-grouped set of entities rather than scanning the whole world.

## Public contract API

| Function | Parameters | Returns | Description |
| --- | --- | --- | --- |
| `init_level` | `player: Address, width: u32, height: u32, tiles: Vec<u32>` | `()` | Validates and loads a level from a row-major tile grid. Requires `player`'s authorization. The same `player` may call this again to replace their level; a different address is rejected if a level is already stored. |
| `push` | `player: Address, direction: u32` | `()` | Moves the player one tile, pushing a crate ahead of it if present. Requires `player`'s authorization and that `player` matches the address that called `init_level`. |
| `level_meta` | - | `LevelMeta` | Move count, solved flag, and level dimensions. |
| `player_position` | - | `Position` | The player's current position. |

### Level encoding

`tiles` is `width * height` entries, row-major (index `y * width + x`), using:

| Constant | Value | Meaning |
| --- | --- | --- |
| `TILE_EMPTY` | 0 | Open floor |
| `TILE_WALL` | 1 | Impassable |
| `TILE_CRATE` | 2 | Pushable crate |
| `TILE_GOAL` | 3 | Crate destination |
| `TILE_PLAYER` | 4 | Player start (exactly one required) |

A level needs exactly one player tile and an equal, positive number of crate and goal tiles, and neither dimension may exceed `MAX_DIM` (20). `push` takes a direction from `NORTH` / `EAST` / `SOUTH` / `WEST` (`+y` / `+x` / `-y` / `-x`), matching the convention `spawn_and_move` uses for the same four constants.

Pushing the player, or a crate it is pushing, outside `0..width` by `0..height` panics with `"out of bounds"`.

## Architecture overview

```
lib.rs        #[contract] entrypoints: validate input, load/save the world, delegate to systems
components.rs Position (re-exported) + Player/Wall/Crate/Goal/OnGoal markers + LevelMeta
systems.rs    try_push (move/push resolution) and is_solved (win check), independent of Soroban glue
test.rs       #[cfg(test)] client-driven tests
```

Entrypoints stay thin: `init_level` and `push` validate input, load state, call into `systems.rs`, persist state, and return. All gameplay logic, including the sole `without_component` query, lives in `systems.rs`. There is no `GameApp`/`ScheduleStage` wiring; a single push is small enough to resolve inline without a scheduled tick.

## Storage model

The entire `ArchetypeWorld` is kept as one value in instance storage, the same tier `impl_soroban_game!` uses for `SimpleWorld` (no equivalent helper macro exists yet for `ArchetypeWorld`, so `lib.rs` wires `load_world`/`save_world` manually around `env.storage().instance()`). The player's `Address` lives under its own instance storage key rather than as a component, because `ArchetypeWorld::get_typed`/`set_typed` are bounded to `ComponentTrait`, and `Address`-bearing data needs `RichComponentTrait` (see `impl_rich_component!`), for which `ArchetypeWorld` has no typed accessors. No persistent or temporary storage is used.

Every call reads and rewrites the whole world blob. That is a deliberate simplification for a single, bounded-size puzzle (`MAX_DIM` keeps grids at 20x20 or smaller), not a pattern to copy for a world with many independent or large levels; see Known limitations.

## Main gameplay flow

1. A caller calls `init_level` with a grid and their own `Address`. The contract validates the grid, spawns one entity per non-empty tile (the player is always entity 1), stores the caller's address, and persists the world.
2. The caller calls `push` with a direction. `systems::try_push` panics if the level is already solved or the move is blocked, otherwise moves the player (and a pushed crate, if any), updates the crate's `OnGoal` marker, increments the move count, and re-runs the win check.
3. Step 2 repeats until every crate carries `OnGoal`, at which point `level_meta().solved` becomes `true` and further `push` calls panic.
4. `level_meta` and `player_position` can be read at any time without mutating state.

## Cougr APIs used

| API | Maturity | Why |
| --- | --- | --- |
| `ArchetypeWorld` | Stable | Entities are grouped by component signature, so `Wall`, `Crate`, `Goal`, `OnGoal`, and `Player` each form their own archetype without any manual indexing. |
| `ArchetypeQueryBuilder` | Stable | `with_component` finds the wall/crate/goal at a tile; `without_component(OnGoal)` on `Crate` is the win check, returning no entities once every crate sits on a goal. |
| `impl_marker_component!` | - | `Player`, `Wall`, `Crate`, `Goal`, `OnGoal` carry no data; a position plus archetype membership is all each one needs. |
| `impl_component!` | - | `LevelMeta` carries real fields (`move_count`, `solved`, `width`, `height`) and uses the same fixed-size serialization every other canonical example's non-marker components use. |

## Build and test commands

```bash
cargo test
stellar contract build
```

## Known limitations

- One level per contract instance, with no multi-level or save-slot support.
- The whole world is read and rewritten on every call. That scales fine for a puzzle-sized grid but is not the storage pattern to copy for a world meant to hold many independent or large levels.
- No state-change events are emitted; `impl_component_observed!` is not used, so clients must poll `level_meta` / `player_position` rather than subscribe to updates.
