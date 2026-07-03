# Snake On-Chain Game

**Classification: Canonical example.** This is the maintained arcade reference for Cougr examples. New arcade contracts should copy its `GameApp` wiring, `components.rs` / `systems.rs` split, README shape, and test coverage.

## Purpose and pattern

Snake demonstrates a deterministic arcade loop on Soroban using `cougr-core`'s basic ECS and `GameApp` tick model. The contract keeps persistent game state on chain, stores ECS entities in a `SimpleWorld`, and runs ordered systems for movement, collision, growth, and food spawning.

## Public contract API

| Function | Parameters | Return type | Description |
|---|---|---:|---|
| `init_game` | none | `()` | Initializes a new game on the default 10×10 grid. |
| `init_game_with_size` | `grid_size: i32` | `()` | Initializes a new game on a custom square grid. |
| `change_direction` | `direction: u32` | `bool` | Changes the snake direction (`0` up, `1` down, `2` left, `3` right); returns `false` for invalid values, reversals, or game-over state. |
| `update_tick` | none | `()` | Advances the game by one `GameApp` tick. |
| `get_score` | none | `u32` | Returns the current score. |
| `check_game_over` | none | `bool` | Returns whether the game has reached a terminal state. |
| `get_head_pos` | none | `(i32, i32)` | Returns the current snake-head position. |
| `get_snake_length` | none | `u32` | Returns the number of snake entities. |
| `get_food_pos` | none | `(i32, i32)` | Returns the current food position. |
| `get_snake_positions` | none | `Vec<(i32, i32)>` | Returns all snake segment positions. |
| `get_grid_size` | none | `i32` | Returns the configured grid size. |

## Architecture overview

```text
contract entrypoint
  ├─ loads GameState + SimpleWorld from persistent storage
  ├─ builds a GameApp around the world
  ├─ schedules systems by stage
  │   ├─ Update: move_snake
  │   └─ PostUpdate: self_collision -> food_collision
  └─ writes GameState + SimpleWorld back to storage
```

- `lib.rs` contains the Soroban contract entrypoints, storage access, and `GameApp` wiring.
- `components.rs` contains serializable ECS components such as `Position`, `DirectionComponent`, `SnakeHead`, `SnakeSegment`, and `Food`.
- `systems.rs` contains reusable game systems for movement, direction validation, collision checks, growth, and food spawning.

## Storage model

| Storage class | Data | Why |
|---|---|---|
| Instance storage | none | The example does not need contract-wide configuration shared across games. |
| Persistent storage | `state: GameState`, `world: SimpleWorld` | Game progress must survive across transactions. `GameState` stores compact scalar data; `SimpleWorld` stores entities and component bytes. |
| Temporary storage | none | No per-ledger cache is needed for deterministic gameplay. |

Within the `SimpleWorld`, dense components such as positions and directions use table-style access, while marker-style components such as food/head/segment are queried as needed.

## Main gameplay flow

1. A player calls `init_game` or `init_game_with_size`.
2. Startup systems spawn the snake head at the grid center and create one food entity.
3. The player calls `change_direction` to submit a valid non-reversing input.
4. The player or a relayer calls `update_tick`.
5. `GameApp` runs movement first, then collision and food checks.
6. A wall/self collision sets `game_over`; eating food grows the snake, increments score, and spawns new food.
7. Query functions expose score, positions, grid size, and terminal state.

## Cougr APIs used

| API | Why it is used |
|---|---|
| `GameApp` | Provides the maintained arcade-loop pattern and owns scheduled system execution per tick. |
| `ScheduleStage` / `SystemConfig` | Ensures movement runs before post-update collision and food systems. |
| `SimpleWorld` | Stores snake, food, and component data in a Soroban-serializable ECS container. |
| `SimpleQueryBuilder` | Scans entities by component type for food, head, and segment queries. |
| `ComponentTrait` | Gives each custom component deterministic serialization and a stable component type. |

This example does not use Cougr auth, privacy, ZK, or standards modules because Snake is intentionally a single-player arcade-loop reference.

## Build and test commands

```bash
cargo test
stellar contract build
```


## Known limitations

**Recommended Testing Approach:**
For comprehensive testing, use the `GameHarness` and `Scenario` APIs provided by `cougr-core`'s `testutils` feature (see [sandbox_tests.rs](src/sandbox_tests.rs)). This allows writing replayable multi-turn scenarios to verify movement trajectories, direction change validation, and tick updates.

**Expected Output:**
```
running 31 tests
test result: ok. 31 passed; 0 failed; 0 ignored
```

### 4. Lint

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

---

## Contract Functions

### Initialization Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `init_game` | - | - | Start with 10×10 grid |
| `init_game_with_size` | `grid_size: i32` | - | Start with custom grid |

### Control Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `change_direction` | `direction: u32` | `bool` | Change movement direction |
| `update_tick` | - | - | Advance game one step |

**Direction Values:**

| Value | Direction | Delta (x, y) |
|-------|-----------|--------------|
| 0 | Up | (0, -1) |
| 1 | Down | (0, +1) |
| 2 | Left | (-1, 0) |
| 3 | Right | (+1, 0) |

### Query Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `get_score` | `u32` | Current score |
| `check_game_over` | `bool` | Game ended status |
| `get_head_pos` | `(i32, i32)` | Head coordinates |
| `get_snake_length` | `u32` | Total length |
| `get_food_pos` | `(i32, i32)` | Food coordinates |
| `get_snake_positions` | `Vec<(i32, i32)>` | All positions |
| `get_grid_size` | `i32` | Grid dimensions |

---

## Deployment

### Testnet Deployment

```bash
# 1. Generate keypair
stellar keys generate --global alice --network <NETWORK>
stellar keys address alice

# 2. Fund account (visit URL with your address)
# https://friendbot.stellar.org/?addr=<YOUR_ADDRESS>

# 3. Deploy contract
stellar contract deploy \
  --wasm target/wasm32v1-none/release/snake.wasm \
  --source alice \
  --network <NETWORK>

# Save the returned Contract ID!
```

### Playing the Game

```bash
CONTRACT_ID="<your-contract-id>"

# Initialize
stellar contract invoke --id $CONTRACT_ID --source alice --network <NETWORK> -- init_game

# Change direction (0=Up, 1=Down, 2=Left, 3=Right)
stellar contract invoke --id $CONTRACT_ID --source alice --network <NETWORK> -- change_direction --direction 0

# Advance game
stellar contract invoke --id $CONTRACT_ID --source alice --network <NETWORK> -- update_tick

# Check score
stellar contract invoke --id $CONTRACT_ID --source alice --network <NETWORK> -- get_score
```

### Deployed Contract

| Network | Contract ID | Explorer |
|---------|-------------|----------|

---

## Project Structure

```
examples/snake/
├── Cargo.toml              # Dependencies (cougr-core, soroban-sdk)
├── README.md               # This documentation
├── .gitignore              # Ignore rules (test_snapshots/, target/)
└── src/
    ├── lib.rs              # Contract entry points (11 functions)
    ├── components.rs       # Components using cougr-core::ComponentTrait
    ├── systems.rs          # Game logic systems
    └── simple_world.rs     # Entity-component storage
```

| File | Purpose |
|------|---------|
| `lib.rs` | Soroban contract with public functions and tests |
| `components.rs` | Component definitions implementing `ComponentTrait` |
| `systems.rs` | Game mechanics (movement, collision, spawning) |
| `simple_world.rs` | Entity and component storage layer |

---

## Creating Components

### Using cougr-core's ComponentTrait

```rust
use cougr_core::component::{Component, ComponentStorage, ComponentTrait};
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

pub struct MyComponent {
    pub value: u32,
}

impl ComponentTrait for MyComponent {
    // Unique identifier for this component type
    fn component_type() -> Symbol {
        symbol_short!("mycomp")
    }

    // Serialize to bytes for on-chain storage
    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.value.to_be_bytes()));
        bytes
    }

    // Deserialize from bytes
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 4 { return None; }
        let value = u32::from_be_bytes([
            data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?
        ]);
        Some(Self { value })
    }

    // Choose storage strategy
    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table  // For dense data
        // ComponentStorage::Sparse  // For marker components
    }
}
```

### Converting to Component

```rust
impl MyComponent {
    pub fn to_component(&self, env: &Env) -> Component {
        Component::new(Self::component_type(), self.serialize(env))
    }
}
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Rust version errors | `rustup update && rustup default stable` |
| WASM target missing | `rustup target add wasm32v1-none` |
| Stellar CLI not found | `brew install stellar-cli` (macOS) |
| Dependency conflicts | `cargo update && cargo clean && cargo build` |
| Test snapshots issues | Delete `test_snapshots/` directory |

### Full Verification

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test && stellar contract build
```

---

## References

| Resource | Link |
|----------|------|
| Soroban Docs | [developers.stellar.org](https://developers.stellar.org/docs/build/smart-contracts) |
| Stellar CLI | [CLI Documentation](https://developers.stellar.org/docs/tools/cli) |
| Cougr Repository | [github.com/salazarsebas/Cougr](https://github.com/salazarsebas/Cougr) |
| Rust Testing | [Rust Book Ch. 11](https://doc.rust-lang.org/book/ch11-00-testing.html) |

---

## License


- Food spawning is deterministic and suitable for examples, not adversarial randomness.
- There is one game state per contract instance.
- No authentication or ownership model is included.
- Rendering and real-time scheduling are out of scope; callers drive ticks through contract invocations.
