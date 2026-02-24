# Flappy Bird

A simple on-chain Flappy Bird clone demonstrating component-based game mechanics using `cougr-core` ECS on Soroban.

The contract showcases:

- Persistent game state management

- Bird physics (gravity and flapping)

- Pipe spawning, movement, and collision detection

- Score tracking and game-over detection

# ECS Architecture
Components

`Position` — stores the `x` and `y` coordinates of an entity (bird or pipe).

`Velocity` — stores the `x` and `y` velocity of an entity.

`BirdState` — stores whether the bird is alive.

`PipeConfig` — stores pipe gap size and center position for scoring/collision.

`PipeMarker` — identifies pipe entities for management (removal when off-screen).

`GameState` — stores score, game-over status, tick count, bird entity ID, and next pipe spawn tick.

Systems / Functions

`init_game(env)` — initializes the world, spawns the bird and initial pipes, and sets up game state.

`flap(env)` — makes the bird jump by setting its vertical velocity.

`update_tick(env)` — applies gravity, updates positions, moves pipes, checks collisions, spawns new pipes, updates score, and removes off-screen pipes.

`get_score(env)` — returns the current game score.

`check_game_over(env)` — returns whether the game has ended.

`get_bird_pos(env)` — returns the current `(x, y)` coordinates of the bird.

`spawn_pipe(world, env, x, gap_center_y)` — helper to spawn a new pipe at the given position.

`remove_offscreen_pipes(world, env)` — helper to remove pipes that have moved off-screen.

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/flappy_bird.wasm --source <secret-key> --network testnet
```

# Contract API
| Function          | Parameters | Description                                                                                        |
| ----------------- | ---------- | -------------------------------------------------------------------------------------------------- |
| `init_game`       | `env: Env` | Initializes the Flappy Bird game world and spawns the bird and pipes.                              |
| `flap`            | `env: Env` | Makes the bird jump (sets vertical velocity).                                                      |
| `update_tick`     | `env: Env` | Advances the game by one tick; updates physics, moves pipes, checks collisions, and updates score. |
| `get_score`       | `env: Env` | Returns the current score.                                                                         |
| `check_game_over` | `env: Env` | Returns whether the game is over.                                                                  |
| `get_bird_pos`    | `env: Env` | Returns the current `(x, y)` coordinates of the bird.                                              |

