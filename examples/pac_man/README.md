# Pac-Man

An on-chain Pac-Man game implemented entirely in Soroban using `cougr-core` ECS patterns.

This contract demonstrates:

- Persistent on-chain game state

- Pac-Man movement with direction control

- Ghost AI with chase and frightened behavior

- Maze management with walls, pellets, and power pellets

- Score tracking, lives, and win/lose conditions

- Event-driven collision system

# ECS Architecture
Components

`Position` — `(x, y)` coordinates for Pac-Man and ghosts.

`Direction` — movement direction for Pac-Man and ghosts.

`GhostMode` — indicates whether a ghost is chasing or frightened.

`CellType` — type of cell in the maze: empty, wall, pellet, or power pellet.

`Ghost` — encapsulates a ghost entity with position, direction, mode, and timers.

`GameState` — stores Pac-Man position, ghosts, maze layout, score, lives, game-over/win status, power mode timer, and last collision events.

Systems / Functions

`init_game(env)` — initializes the maze, places Pac-Man and ghosts, counts pellets, and stores the initial game state.

`change_direction(env, direction)` — updates Pac-Man’s direction for the next tick.

`update_tick(env)` — main game loop: moves Pac-Man, collects pellets, moves ghosts, checks collisions, updates timers, and checks win condition.

`eat_pellet(env)` — manually eat a pellet at Pac-Man’s position (mainly handled automatically by `update_tick`).

`get_score(env)` — returns current score.

`get_lives(env)` — returns remaining lives.

`get_pacman_position(env)` — returns Pac-Man’s current `(x, y)` coordinates.

`get_maze(env)` — returns the current maze state.

`get_game_state(env)` — returns full game state including ghosts and collision events.

`check_game_over(env)` — returns `(game_over, won)`.

`get_collision_events(env)` — returns collision events from the latest tick.

`get_pacman_core_position(env)` — returns Pac-Man’s position as a `cougr_core` Position component.

`get_serialized_pacman_position(env)` — returns Pac-Man’s position serialized with ComponentTrait.

Internal Helpers

`move_pacman(state)` — updates Pac-Man’s position according to current direction, handles wall collisions and maze wrapping.

`check_pellet_collection(state)` — handles pellet and power pellet consumption and activates frightened mode for ghosts.

`move_ghosts(state)` — updates ghost positions using simple AI based on chase/frightened mode.

`calculate_ghost_direction(state, ghost, pacman_pos)` — chooses the best movement direction for a ghost.

`check_ghost_collisions(state)` — handles collisions between Pac-Man and ghosts.

`activate_power_mode(state)` — sets all ghosts to frightened mode for POWER_MODE_DURATION.

`end_frightened_mode(state)` — reverts ghosts back to chase mode.

`create_maze(env)` — generates the initial 10x10 maze with walls, pellets, and power pellets.

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/pac_man.wasm --source <secret-key> --network testnet
```

Contract API
| Function                         | Parameters                       | Description                                                         |
| -------------------------------- | -------------------------------- | ------------------------------------------------------------------- |
| `init_game`                      | `env: Env`                       | Initializes the Pac-Man game and sets up maze, Pac-Man, and ghosts. |
| `change_direction`               | `env: Env, direction: Direction` | Updates Pac-Man's movement direction.                               |
| `update_tick`                    | `env: Env`                       | Advances the game by one tick (main game loop).                     |
| `eat_pellet`                     | `env: Env`                       | Eats a pellet at Pac-Man's current position.                        |
| `get_score`                      | `env: Env`                       | Returns current score.                                              |
| `get_lives`                      | `env: Env`                       | Returns remaining lives.                                            |
| `get_pacman_position`            | `env: Env`                       | Returns Pac-Man’s `(x, y)` coordinates.                             |
| `get_maze`                       | `env: Env`                       | Returns the maze layout.                                            |
| `get_game_state`                 | `env: Env`                       | Returns complete game state.                                        |
| `check_game_over`                | `env: Env`                       | Returns `(game_over, won)`.                                         |
| `get_collision_events`           | `env: Env`                       | Returns collision events from the last tick.                        |
| `get_pacman_core_position`       | `env: Env`                       | Returns Pac-Man’s position as a `cougr_core` Position.              |
| `get_serialized_pacman_position` | `env: Env`                       | Returns Pac-Man’s serialized position via `ComponentTrait`.         |
