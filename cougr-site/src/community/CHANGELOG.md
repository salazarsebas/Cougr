# Changelog

## 1.1.0

### Added

- **`game::SorobanGame` trait** — standard `load_world` / `save_world` contract pattern;
  implement once with `impl_soroban_game!(Contract, "key")`, use in every entrypoint
- **`impl_soroban_game!` macro** — wires `SorobanGame` to any `#[contract]` struct
- **`SimpleWorld::load_from_instance`** — load world from Soroban instance storage,
  returning a fresh empty world on first call
- **`SimpleWorld::save_to_instance`** — persist world to Soroban instance storage
- **`SimpleWorld::set_rich_observed`** — store a rich component and emit a
  `RichComponentChangedEvent` for off-chain indexers
- **`SimpleWorld::remove_rich_observed`** — remove a rich component and emit a `del` event
- **`RichComponentChangedEvent`** — new Soroban event type with topics
  `("COUGR", "rich", component_type)` for rich component change notifications
- **`spawn_and_move` example** — canonical Cougr starter game demonstrating the complete
  idiomatic pattern: `impl_component_observed!` + `SorobanGame` + typed ECS access
- **`SorobanGame` re-exported from `prelude`** — import from `cougr_core::prelude::*`
- **`cougr_core::circuits`** — four pre-built ZK game builders (hidden cards, fog of war,
  fair dice, sealed bid) with pipeline-embedded verification keys
- **`cougr_core::session`** — `SessionManager`, `SessionStatus`, and `ActiveSession` (Beta)
- **`cougr_core::test`** — `GameHarness`, `Scenario`, and `ReplayLog` sandbox behind the
  `testutils` feature
- **Circom pipeline** — `internal/cougr-core-circuits` with CI workflow and on-chain Groth16
  proof verification using real VKs
- **ZK examples** — `hidden_hand`, `fog_explorer`, `dice_duel`, and `blind_auction`
- **Workspace subcrates** — `internal/cougr-core-{circuits,session,test}` per ADR 0007

### Changed

- `tic_tac_toe` example modernised: replaced ~200 lines of manual serialization with
  `impl_rich_component!` for `Board` and `Players`, and `impl_soroban_game!` for
  load/save. Public API is unchanged; all existing tests pass
- README rewritten with clean 30-line quick start and full feature documentation
- canonical example set expanded from three (`snake`, `battleship`, `guild_arena`) to ten:
  `spawn_and_move` (Starter), `tic_tac_toe` (Rich components), `session_arena` (Session UX),
  `hidden_hand`, `fog_explorer`, `dice_duel`, `blind_auction` (ZK circuits),
  `snake` (Arcade/GameApp), `battleship` (Hidden information), `guild_arena` (Auth & recovery)
- `session_arena` example added as canonical reference for `session::SessionManager`

### Stability Notes

- `game::SorobanGame` is **Stable**
- `SimpleWorld::load_from_instance` / `save_to_instance` are **Stable**
- `set_rich_observed` / `remove_rich_observed` are **Stable**
- `RichComponentChangedEvent` is **Stable**
- `cougr_core::session` is **Beta**
- `cougr_core::circuits` and embedded test VKs are **Experimental**
- `cougr_core::test` is **Experimental** (`testutils` only)

---

## 1.0.0

### Added

- `app` as the default gameplay runtime surface
- `auth`, `privacy`, and `ops` as product-level domain namespaces
- `RuntimeWorld` and `RuntimeWorldMut` as shared Soroban-first backend contracts
- stronger stage scheduling with ordering, sets, and validation
- `SimpleQueryBuilder`, query state/cache improvements, and richer `ArchetypeWorld` query helpers
- expanded benchmark coverage for backend comparisons and cache invalidation behavior

### Changed

- the recommended onboarding path is now `app::GameApp` + `SimpleWorld` + `SimpleQueryBuilder`
- canonical examples now emphasize the curated runtime story and explicit maturity boundaries
- `battleship` now uses stable privacy primitives from `zk::stable`
- documentation now treats `SimpleWorld` and `ArchetypeWorld` as the defended Soroban-first backends

### Stability Notes

- Stable: ECS onboarding/runtime contract, `app`, `ops`, `standards`, `privacy::stable`, `zk::stable`
- Beta: `auth`, `accounts`, `game_world`
- Experimental: `privacy::experimental`, `zk::experimental`, hazmat cryptographic helpers

### Upgrade Notes

- Prefer `app` over wiring scheduler/world primitives directly for new gameplay code
- If you still have pre-1.0 code built around removed runtime abstractions, port directly to `GameApp`, `SimpleWorld`, and `SimpleQuery`
- Prefer `ops`, `privacy`, and `auth` in application code when you want domain-oriented imports
- Treat root-level advanced re-exports as compatibility/advanced surfaces rather than the default learning path
- See [docs/MIGRATION_GUIDE.md](../reference/MIGRATION_GUIDE.md) for concrete migration mappings
