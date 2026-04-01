# Tower Defense Example

A minimal tower defense game implemented as a Soroban smart contract using the `cougr-core` ECS framework. This example demonstrates wave spawning, enemy path progression, tower attacks, health reduction, and win/loss conditions.

## Features

- **Wave Spawning**: Enemies spawn in waves with increasing difficulty
- **Deterministic Path Progression**: Enemies follow a predefined path toward the base
- **Tower Placement**: Three tower types with different stats (Basic, Sniper, Splash)
- **Attack Resolution**: Towers automatically target and damage enemies in range
- **Win/Loss Conditions**: Survive all waves to win, or lose when base health reaches 0

## ECS Architecture

### Components

| Component | Fields | Purpose |
|-----------|--------|---------|
| `EnemyComponent` | hp, max_hp, speed, path_index | Represents enemies moving along the path |
| `TowerComponent` | kind, range, damage, cooldown | Represents static defenders |
| `WaveComponent` | current_wave, total_waves, remaining_spawns | Tracks wave progression |
| `BaseComponent` | health, max_health | Tracks survival condition |
| `GameStatusComponent` | status, tick_count, enemies_killed | Tracks game state |

### Systems

- **WaveSpawnSystem**: Spawns enemies according to wave configuration
- **PathProgressionSystem**: Moves enemies along the predefined path
- **AttackResolutionSystem**: Towers target and damage enemies in range
- **BaseDamageSystem**: Reduces base health when enemies reach the end
- **EndConditionSystem**: Checks win/loss conditions

## Contract API

### `fn init_game(env: Env)`
Initializes a new game with default settings.

### `fn place_tower(env: Env, x: u32, y: u32, tower_kind: u32) -> bool`
Places a tower at the specified coordinates.
- `tower_kind`: 0=Basic, 1=Sniper, 2=Splash
- Returns `true` if placement was successful

### `fn advance_tick(env: Env)`
Advances the game by one tick, executing all game systems.

### `fn get_state(env: Env) -> GameState`
Returns the current game state including base health, wave info, and status.

### `fn is_finished(env: Env) -> bool`
Returns `true` if the game has ended (won or lost).

### `fn get_result(env: Env) -> u32`
Returns the game result: 0=active, 1=won, 2=lost.

## Setup

1. Navigate to the example directory:
   ```bash
   cd examples/tower_defense
   ```

2. Build the contract:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Validation Commands

```bash
cd examples/tower_defense
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```

## Game Configuration

- **Map Size**: 10x10 grid
- **Path Length**: 8 waypoints
- **Total Waves**: 5
- **Enemies per Wave**: 5
- **Base Health**: 100
- **Enemy Base Damage**: 10

### Tower Stats

| Tower | Range | Damage | Cooldown |
|-------|-------|--------|----------|
| Basic | 2 | 10 | 1 tick |
| Sniper | 4 | 25 | 3 ticks |
| Splash | 1 | 15 | 2 ticks |