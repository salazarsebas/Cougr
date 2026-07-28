# Testing Guide: GameHarness, Scenarios, and Snapshot Assertions

This guide teaches how to test Cougr on-chain game logic using `cougr_core::test`. 

In Cougr, testing is a first-class developer experience. Rather than writing manual, repetitive setup scripts or relying on heavyweight full-node deployments, Cougr provides an in-memory, deterministic sandbox environment (`cougr_core::test`, enabled via the `testutils` feature) designed specifically for testing Entity Component System (ECS) game logic, multi-player turn sequences, and state snapshots on Soroban.

---

## Architecture Overview

The `cougr_core::test` module provides five core abstractions for testing game contracts:

| Component | Purpose |
|---|---|
| [`GameHarness`](#1-setting-up-a-gameharness) | Registers contracts in a Soroban `Env`, manages mock player addresses, and scopes contract execution. |
| [`Scenario`](#2-writing-turn-based-scenarios-scenario) | Declarative driver for multi-player / multi-turn gameplay loops with automatic player rotation. |
| [`WorldFixture`](#3-seeding-and-injecting-state-worldfixture) | Constructs or reads `SimpleWorld` instances and injects/restores them into live contract storage. |
| [`SnapshotAssert`](#4-asserting-world-behavior-snapshotassert) | Lightweight, `no_std`-compatible assertions for entity counts, world versions, and state diffing. |
| [`ReplayLog`](#5-recording-and-forking-state-replaylog) | Append-only state checkpoint log used to compare world state across turns and fork mid-game for debugging. |

---

## 1. Setting Up a `GameHarness`

`GameHarness` is the entry point for every integration test. It wraps a Soroban `Env` and a registered contract instance, keeping track of contract credentials and registered player addresses.

### Basic Initialization

```rust
use cougr_core::test::GameHarness;
use soroban_sdk::Env;
use crate::MyGameContract;

let env = Env::default();
let harness = GameHarness::new(env, MyGameContract);

// Access the environment and contract ID
let env_ref = harness.env();
let contract_id = harness.contract_id();
```

If your contract is already registered in the `Env` (for example, as part of a complex multi-contract setup), use `from_registered`:

```rust
let harness = GameHarness::from_registered(env, contract_id);
```

### Mocking Players and Authorization

Game tests frequently require multiple player identities. `GameHarness` can generate mock `Address` instances and mock authentication:

```rust
let mut harness = GameHarness::new(Env::default(), MyGameContract);

// Generate 4 mock player addresses
let players = harness.mock_players(4);

// Access player by slot (PlayerSlot(0), PlayerSlot(1), ...)
let player_1 = harness.player(cougr_core::test::PlayerSlot(0));

// Automatically mock Soroban authorizations for all mock players
harness.mock_all_auths();
```

### Executing Code in Contract Context

When direct access to contract instance storage or contract functions is required without invoking the generated client, wrap your logic with `as_contract`:

```rust
harness.as_contract(|| {
    // Code executed inside the registered contract's execution scope
});
```

---

## 2. Writing Turn-Based Scenarios (`Scenario`)

`Scenario` provides a declarative builder for testing multi-player, multi-turn game flows. It automatically handles turn indexing and rotates player slots across registered players.

### Basic Scenario Execution

```rust
use cougr_core::test::{GameHarness, Scenario};
use soroban_sdk::Env;

let mut harness = GameHarness::new(Env::default(), MyGameContract);
harness.mock_players(2); // PlayerSlot(0) and PlayerSlot(1)

Scenario::new("alternating movement scenario")
    .players(2) // 2 players
    .turns(4)   // 4 turns total: turn 0 -> P0, turn 1 -> P1, turn 2 -> P0, turn 3 -> P1
    .run(&harness, |player_slot, turn_index, h| {
        let player_addr = h.player(player_slot);
        let client = MyGameContractClient::new(h.env(), h.contract_id());
        
        // Execute player action for this turn
        client.take_turn(player_addr, &turn_index.0);
    });
```

### `run_and_assert`

Use `run_and_assert` to drive a sequence of turns and immediately run assertion logic against the final state:

```rust
Scenario::new("spawn phase")
    .players(1)
    .turns(3)
    .run_and_assert(
        &harness,
        |_player, _turn, h| {
            MyGameContractClient::new(h.env(), h.contract_id()).spawn();
        },
        |h| {
            // Evaluated once after all 3 turns complete
            let client = MyGameContractClient::new(h.env(), h.contract_id());
            assert_eq!(client.entity_count(), 3);
        },
    );
```

---

## 3. Seeding and Injecting State (`WorldFixture`)

`WorldFixture` allows you to pre-populate an ECS world off-chain and inject it directly into the contract's storage, bypassing step-by-step transaction setup.

### Creating and Injecting Fixtures

```rust
use cougr_core::test::{GameHarness, WorldFixture};
use cougr_core::component::ComponentTrait;
use soroban_sdk::Env;
use crate::{MyGameContract, Position};

let harness = GameHarness::new(Env::default(), MyGameContract);

// Build an empty fixture
let mut fixture = WorldFixture::empty(harness.env());

// Spawn entities and attach components off-chain
let entity_1 = fixture.spawn_entity();
fixture.set_typed(harness.env(), entity_1, &Position { x: 10, y: 20 });

// Inject the pre-built world directly into contract storage
fixture.inject::<MyGameContract>(&harness);
```

### Reading World State from Storage

You can read the live `SimpleWorld` out of contract storage into a `WorldFixture` at any point during testing:

```rust
let live_fixture = WorldFixture::read_from_contract::<MyGameContract>(&harness);
let world = live_fixture.world();

assert_eq!(live_fixture.entity_count(), 1);
```

---

## 4. Asserting World Behavior (`SnapshotAssert`)

`SnapshotAssert` provides zero-allocation, `no_std`-friendly assertions for validating ECS world states.

### Entity and Version Assertions

```rust
use cougr_core::test::{SnapshotAssert, WorldFixture};

let fixture = WorldFixture::read_from_contract::<MyGameContract>(&harness);

// Assert exact entity count
SnapshotAssert::assert_entity_count(fixture.world(), 3);

// Compare world versions before and after a system run
let before_world = fixture.world().clone();

// ... perform contract actions ...

let after_fixture = WorldFixture::read_from_contract::<MyGameContract>(&harness);
SnapshotAssert::assert_version_increased(&before_world, after_fixture.world());
```

### Diffing State Deltas (`debug` Feature)

When the `debug` feature flag is enabled in `cougr-core`, `SnapshotAssert::diff_entity_delta` can calculate total structural modifications (added/removed entities and added/removed/modified components) between two snapshots:

```rust
#[cfg(feature = "debug")]
{
    let delta_count = SnapshotAssert::diff_entity_delta(
        harness.env(),
        &before_world,
        after_fixture.world(),
    );
    assert!(delta_count > 0, "expected world modifications");
}
```

---

## 5. Recording and Forking State (`ReplayLog`)

`ReplayLog` records world snapshots turn-by-turn during a scenario run. It is particularly useful for debugging complex game failures or testing divergent choices from a common game state.

### Recording Checkpoints

```rust
use cougr_core::test::{GameHarness, ReplayLog, TurnIndex};

let harness = GameHarness::new(Env::default(), MyGameContract);
let client = MyGameContractClient::new(harness.env(), harness.contract_id());

let mut log = ReplayLog::new();

// Turn 0: spawn entity
client.spawn();
log.record::<MyGameContract>(TurnIndex(0), &harness);

// Turn 1: spawn another entity
client.spawn();
log.record::<MyGameContract>(TurnIndex(1), &harness);

// Assert world state changed between turn 0 and turn 1
log.assert_differs_at(0, 1);
```

### Forking Mid-Game to Debug

If turn 10 fails, you don't need to rebuild the scenario state manually. You can restore contract storage directly to the snapshot captured at turn 9 and test alternative actions:

```rust
// Restore contract state back to how it was at turn 0
let forked_fixture = log.fork_from::<MyGameContract>(&harness, 0);

// Verify contract storage was updated to match the checkpoint
assert_eq!(forked_fixture.entity_count(), 1);
```

---

## 6. Snapshot Testing Conventions

Cougr projects use standardized file layout and conventions for snapshot testing. Following these conventions ensures consistent test output across all game crates.

### Directory Structure

```text
my_game/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── components.rs
│   ├── systems.rs
│   └── test.rs
└── test_snapshots/             # Generated test snapshot artifacts
    └── scenario_battle_turn_3.json
```

### Guidelines for Snapshot Tests

1. **Ignore Transient Artifacts**: Ensure `test_snapshots/` is listed in your project `.gitignore` if snapshots contain build-environment dependent paths or non-deterministic timestamps. If snapshots are intended as checked-in golden files, verify they are clean and deterministic before committing.
2. **Deterministic Inputs**: Always use standard deterministic keys or `Env::default()` seed values during snapshot runs.
3. **Clean Up Ephemeral Snapshots**: When debugging failing tests, purge stale snapshots if schema changes alter component representations:
   ```bash
   rm -rf test_snapshots/
   cargo test
   ```

---

## 7. Fully Worked Example

Below is a complete, copy-pasteable, self-contained integration test module demonstrating all `cougr_core::test` primitives in action.

```rust
#[cfg(test)]
mod tests {
    use cougr_core::component::ComponentTrait;
    use cougr_core::game::SorobanGame;
    use cougr_core::test::{
        GameHarness, ReplayLog, Scenario, SnapshotAssert, TurnIndex, WorldFixture,
    };
    use cougr_core::{impl_component, impl_soroban_game};
    use soroban_sdk::{contract, contractimpl, contracttype, Env};

    // 1. Define component
    #[contracttype]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Position {
        pub x: i32,
        pub y: i32,
    }
    impl_component!(Position, "pos", Table, { x: i32, y: i32 });

    // 2. Define game contract
    #[contract]
    #[derive(Clone)]
    pub struct BattleArena;

    impl_soroban_game!(BattleArena, "world");

    #[contractimpl]
    impl BattleArena {
        pub fn spawn(env: Env) -> u32 {
            let mut world = BattleArena::load_world(&env);
            let entity = world.spawn_entity();
            world.set_typed(&env, entity, &Position { x: 0, y: 0 });
            BattleArena::save_world(&env, &world);
            entity
        }

        pub fn move_right(env: Env, entity: u32) {
            let mut world = BattleArena::load_world(&env);
            if let Some(mut pos) = world.get_typed::<Position>(&env, entity) {
                pos.x += 1;
                world.set_typed(&env, entity, &pos);
                BattleArena::save_world(&env, &world);
            }
        }

        pub fn entity_count(env: Env) -> u32 {
            BattleArena::load_world(&env)
                .next_entity_id()
                .saturating_sub(1)
        }
    }

    #[test]
    fn test_full_sandbox_flow() {
        let env = Env::default();
        let mut harness = GameHarness::new(env, BattleArena);

        // A. Setup mock players and auths
        harness.mock_players(2);
        harness.mock_all_auths();

        // B. Seed state using WorldFixture
        let mut fixture = WorldFixture::empty(harness.env());
        let preseeded_id = fixture.spawn_entity();
        fixture.set_typed(harness.env(), preseeded_id, &Position { x: 10, y: 10 });
        fixture.inject::<BattleArena>(&harness);

        let client = BattleArenaClient::new(harness.env(), harness.contract_id());
        assert_eq!(client.entity_count(), 1);

        // C. Run a multi-turn Scenario and record ReplayLog
        let mut replay_log = ReplayLog::new();

        Scenario::new("arena battle sequence")
            .players(2)
            .turns(3)
            .run(&harness, |_player, turn, h| {
                let c = BattleArenaClient::new(h.env(), h.contract_id());
                c.spawn();
                c.move_right(&preseeded_id);

                // Snapshot turn state into replay log
                replay_log.record::<BattleArena>(turn, h);
            });

        // Total entities: 1 preseeded + 3 spawned in scenario = 4
        assert_eq!(client.entity_count(), 4);
        assert_eq!(replay_log.len(), 3);

        // D. Assertions with SnapshotAssert
        let current_fixture = WorldFixture::read_from_contract::<BattleArena>(&harness);
        SnapshotAssert::assert_entity_count(current_fixture.world(), 4);

        // Check that turn 0 state differs from turn 2 state
        replay_log.assert_differs_at(0, 2);

        // E. Fork mid-game back to turn 0 checkpoint
        let turn_0_fixture = replay_log.fork_from::<BattleArena>(&harness, 0);
        SnapshotAssert::assert_entity_count(turn_0_fixture.world(), 2);
    }
}
```

---

## Related Documentation

- [ARCHITECTURE.md](../../ARCHITECTURE.md) — Cougr core module design
- [docs/ECS_CORE.md](../ECS_CORE.md) — ECS runtime mechanics
- [docs/PATTERNS.md](../PATTERNS.md) — Common gameplay implementation patterns
