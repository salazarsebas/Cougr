# Cougr Game Logic Testing Guide

This guide details recommended testing patterns using `cougr_core::test` test utilities (`GameHarness`, `Scenario`, `WorldFixture`, `ReplayLog`, `SnapshotAssert`).

## 🛠️ Testing Core Concepts

### GameHarness
`GameHarness` manages simulated Soroban environment execution, registering system contracts, setting up mock block ledgers, and driving deterministic ticks.

### Scenario & WorldFixture
- **WorldFixture**: Pre-populates the ECS world with entities, components, and resource balances prior to test execution.
- **Scenario**: Defines sequential user transaction steps and expected state assertions.

```rust
use cougr_core::testutils::{GameHarness, Scenario, WorldFixture};

#[test]
fn test_spawn_and_move_scenario() {
    let mut harness = GameHarness::new();
    let fixture = WorldFixture::default()
        .with_player("player_1")
        .with_position(0, 0);

    harness.load_fixture(fixture);

    let scenario = Scenario::new()
        .step("move_east", |h| h.invoke_system("move", (1, 0)))
        .assert_position("player_1", 1, 0);

    scenario.run(&mut harness);
}
```

### SnapshotAssert & ReplayLog
- **SnapshotAssert**: Compares current world component states against saved golden snapshot files (`test_snapshots/*.snap`).
- **ReplayLog**: Records input transaction traces to reproduce and debug failing scenarios.
