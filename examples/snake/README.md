# Snake

Classic Snake game implemented on-chain using the `cougr-core` ECS framework on the Stellar blockchain via Soroban.
Demonstrates entity-component-system patterns for snake movement, food spawning, collision detection, and scoring.

# ECS Architecture
Components

`Position` — Stores `x, y` coordinates of entities (snake head, segments, food)

`DirectionComponent` — Stores current movement direction of the snake head

`SnakeHead` — Marks an entity as the snake’s head

`SnakeSegment` — Marks an entity as a snake body segment with an index

`Food` — Marks an entity as food

`GameState` — Stores score, game over status, tick count, grid size, and snake head ID

Systems

`move_snake()` — Moves the snake in its current direction

`update_direction()` — Changes the snake's movement direction (prevents reversing)

`check_self_collision()` — Ends game if snake collides with itself

`check_food_collision()` — Detects food collision and triggers growth

`grow_snake()` — Adds a new segment to the snake after eating food

`spawn_food()` — Places new food on the grid

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/snake.wasm --source <secret-key> --network testnet
```
# Contract API
| Function              | Parameters            | Description                                                                              |
| --------------------- | --------------------- | ---------------------------------------------------------------------------------------- |
| `init_game`           | `env`                 | Initializes a new game with default 10x10 grid                                           |
| `init_game_with_size` | `env, grid_size: i32` | Initializes a new game with a custom grid size                                           |
| `change_direction`    | `env, direction: u32` | Changes snake movement direction; 0=Up, 1=Down, 2=Left, 3=Right                          |
| `update_tick`         | `env`                 | Advances the game by one tick: moves snake, checks collisions, grows snake if food eaten |
| `get_score`           | `env`                 | Returns the current score                                                                |
| `check_game_over`     | `env`                 | Returns true if game is over, false otherwise                                            |
| `get_head_pos`        | `env`                 | Returns (x, y) position of snake head                                                    |
| `get_snake_length`    | `env`                 | Returns current snake length including head and segments                                 |
| `get_food_pos`        | `env`                 | Returns (x, y) position of food, (-1, -1) if none exists                                 |
| `get_snake_positions` | `env`                 | Returns positions of all snake entities in order (head first)                            |
| `get_grid_size`       | `env`                 | Returns the current grid size                                                            |


