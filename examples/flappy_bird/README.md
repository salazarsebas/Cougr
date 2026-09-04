# Flappy Bird On-Chain Game

> **Transitional example**: This example uses an older Cougr pattern and is preserved
> for compatibility reference. For the current recommended approach, see `snake`.

## Purpose and pattern

This example demonstrates a side-scroller gravity loop on Soroban with Cougr ECS concepts. It remains transitional while the arcade examples converge on the canonical `snake` `GameApp` architecture.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | `none` | `()` | Initializes bird, velocity, pipes, score, and tick state. |
| `flap` | `none` | `()` | Applies upward input velocity if the game is still active. |
| `update_tick` | `none` | `()` | Runs one scheduled tick for gravity, movement, pipes, collisions, and scoring. |
| `get_score` | `none` | `u32` | Returns the current score. |
| `check_game_over` | `none` | `bool` | Returns whether the bird has crashed. |
| `get_bird_pos` | `none` | `(i32, i32)` | Returns the bird position. |

## Architecture overview

```text
contract entrypoint
  ├─ reads game state from Soroban storage
  ├─ applies input or tick systems
  └─ writes updated state back to storage
```

Bird, pipe, position, velocity, and scoring markers are represented as ECS components in `components.rs` and updated by systems in `systems.rs`.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | Per-contract game state where used by this example. | Keeps small arcade state close to the contract instance. |
| Persistent storage | Player- or world-scoped state where the example needs durable keyed state. | Keeps game progress available across invocations. |
| Temporary storage | Not used. | The examples favor deterministic recalculation over ephemeral caches. |

## Main gameplay flow

1. Call the initialization function to create the starting state.
2. Submit an input action such as movement, jump, flap, rotation, or shoot.
3. Call the tick/update function to run deterministic simulation logic.
4. Query public getters for score, position, active state, or terminal status.
5. Stop when the game-over/completed condition is reached, or reset/reinitialize where supported.

## Cougr APIs used

- `ComponentTrait` and custom component modules document the ECS data boundary.
- `SimpleWorld`, `SimpleQueryBuilder`, `GameApp`, `ScheduleStage`, or `SystemConfig` are used where this transitional example has already adopted the maintained runtime shape.
- Auth, privacy, ZK, and standards APIs are intentionally not used; these arcade examples focus on deterministic game logic.

## Build and test commands

```bash
cargo test
stellar contract build
```


## Known limitations

The tests cover:
- Game initialization
- Bird flapping mechanics
- Gravity and physics
- Collision detection
- Score tracking
- Game over conditions

## Deploying to Testnet

### 1. Create and Fund Account

Generate a new keypair:

```bash
stellar keys generate test-account --network <NETWORK>
```

Get the account address:

```bash
stellar keys address test-account
```

Fund the account using Friendbot:

```bash
curl "https://friendbot.stellar.org?addr=$(stellar keys address test-account)"
```

### 2. Deploy Contract

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/flappy_bird.wasm \
  --source test-account \
  --network <NETWORK>
```

Save the returned CONTRACT_ID for later use.

### 3. Play the Game

#### Initialize Game

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- init_game
```

#### Make Bird Flap

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- flap
```

#### Advance Game Tick

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- update_tick
```

#### Check Score

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- get_score
```

#### Check Game Status

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- check_game_over
```

#### Get Bird Position

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source test-account \
  --network <NETWORK> \
  -- get_bird_pos
```

## Game Mechanics

### Constants

- `GRAVITY`: 2 pixels per tick
- `FLAP_VELOCITY`: -15 pixels per tick (upward)
- `PIPE_SPEED`: 3 pixels per tick (leftward)
- `GROUND_Y`: 400 pixels
- `BIRD_SIZE`: 20 pixels (hitbox radius)
- `PIPE_GAP`: 100 pixels

### How to Play

1. Initialize a new game with `init_game()`
2. Make the bird flap with `flap()` to gain upward velocity
3. Advance the game with `update_tick()` - this applies:
   - Gravity to the bird
   - Movement to all entities
   - Collision detection
   - Score updates
4. Repeat steps 2-3 to keep the bird alive and score points
5. Game ends when bird hits ground, ceiling, or pipes

### Scoring

- Score increases by 1 for each pipe successfully passed
- Bird must pass through the gap without touching the pipe

## Code Examples

### Creating a Custom Component

```rust
#[contracttype]
#[derive(Clone, Debug)]
pub struct BirdState {
    pub is_alive: bool,
}

impl ComponentTrait for BirdState {
    fn component_type() -> Symbol {
        symbol_short!("birdstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let value: u8 = if self.is_alive { 1 } else { 0 };
        bytes.append(&Bytes::from_array(env, &[value]));
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let is_alive = data.get(0).unwrap() != 0;
        Some(Self { is_alive })
    }
}
```

### Implementing a Game System

```rust
pub fn apply_gravity(world: &mut World, env: &Env) {
    for entity in world.entities.iter_entities() {
        let entity_id = entity.id();
        if entity.has_component(&symbol_short!("birdstate")) {
            if let Some(vel_comp) = world.get_component(entity_id, &symbol_short!("velocity")) {
                if let Some(mut velocity) = Velocity::deserialize(env, vel_comp.data()) {
                    velocity.y += GRAVITY;
                    let new_vel_comp = Component::new(
                        symbol_short!("velocity"),
                        velocity.serialize(env)
                    );
                    world.storage.add_component(entity_id, new_vel_comp);
                }
            }
        }
    }
}
```

## Troubleshooting

### Common Issues

**Issue**: `error: failed to compile`
- **Solution**: Ensure you have the correct Rust version (1.88.0+) and wasm32 target installed
- Run: `rustup update && rustup target add wasm32v1-none`

**Issue**: `stellar: command not found`
- **Solution**: Install Stellar CLI following the [official guide](https://developers.stellar.org/docs/tools/cli/install)

**Issue**: Contract size too large
- **Solution**: Ensure you're using release mode and the optimizations in Cargo.toml:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  ```

**Issue**: Transaction fails with "budget exceeded"
- **Solution**: On-chain games are resource-intensive. Consider simplifying game logic or reducing the number of entities

### Debugging Tips

1. Use `cargo test` to verify logic before deploying
2. Use `--simulate` flag to test transactions without broadcasting:
   ```bash
   stellar contract invoke --id <CONTRACT_ID> --simulate -- init_game
   ```
3. Check contract logs in Stellar Explorer
4. Use smaller test scenarios to isolate issues

## Performance Considerations

### On-Chain Constraints

- **WASM Binary Size**: Keep contract under 1MB (optimized binary should be ~200KB)
- **Transaction Costs**: Each transaction consumes fees - minimize storage operations
- **Execution Budget**: Soroban has CPU and memory limits - keep game ticks simple

### Optimizations Used

1. **Opt-level "z"**: Optimize for size in release builds
2. **LTO**: Link-time optimization reduces binary size
3. **Minimal Storage**: Only store essential game state
4. **Efficient Queries**: Cache entity lookups when possible

## Next Steps

### Extending the Game

1. **Add Power-ups**: Create new components for temporary abilities
2. **Difficulty Scaling**: Increase pipe speed over time
3. **Multiplayer**: Store multiple game states with player IDs
4. **Leaderboard**: Track high scores across players
5. **Visual Output**: Create a frontend that reads contract state and renders graphics

### Learning More

- [Cougr-Core Documentation](../../README.md)
- [Soroban Documentation](https://developers.stellar.org/docs/smart-contracts)
- [ECS Pattern](https://en.wikipedia.org/wiki/Entity_component_system)

## License

This example is part of the Cougr project and follows the same license.

## Contributing


- Transitional code may preserve older storage or scheduling patterns for compatibility reference.
- No authentication, matchmaking, real-time rendering, or production randomness is included.
- One contract instance generally represents one game or one keyed set of player games.
- For new work, prefer the canonical `snake` module split and `GameApp` tick wiring.
