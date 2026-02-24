# Space Invaders

Classic Space Invaders game implemented on-chain using the `cougr-core` ECS framework on the Stellar blockchain via Soroban.
Demonstrates entity-component-system patterns for player ship control, invader movement, shooting, collision detection, and scoring.

# ECS Architecture
Components

`Position` — Tracks X/Y coordinates of all entities (ship, invaders, bullets)

`Velocity` — Tracks movement speed and direction for bullets and invaders

`Health` — Tracks entity health (ship or invader)

`Ship` — Marks the player ship entity

`Invader` — Marks an invader entity with type and active status

`Bullet` — Marks a bullet fired by player or invaders

`GameState` — Stores score, remaining lives, tick counter, invader direction, and game over status

Systems

`Movement System` — Updates positions of bullets and invaders each tick

`Collision System` — Detects collisions between bullets and entities, applying damage and updating Health components

`Invader Movement System` — Moves invaders horizontally, reverses direction at edges, and moves them down when reaching screen bounds

`Shooting System` — Manages firing cooldowns and spawns bullets for player and invaders

`Game Logic` — Checks win/loss conditions and updates game over state

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/space_invaders.wasm --source <secret-key> --network testnet
```

# Contract API
| Function              | Parameters            | Description                                                                                                                  |
| --------------------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `init_game`           | `env`                 | Initializes a new game, spawns ship and invader entities in ECS World                                                        |
| `move_ship`           | `env, direction: i32` | Moves player ship left (-1) or right (1) within screen bounds                                                                |
| `shoot`               | `env`                 | Fires a bullet from the player ship, obeying cooldown; returns success boolean                                               |
| `update_tick`         | `env`                 | Advances the game by one tick: moves entities, checks collisions, updates health, manages invader wave, spawns enemy bullets |
| `get_score`           | `env`                 | Returns current score                                                                                                        |
| `get_lives`           | `env`                 | Returns remaining lives of the player ship                                                                                   |
| `get_ship_position`   | `env`                 | Returns X position of player ship                                                                                            |
| `check_game_over`     | `env`                 | Returns true if the game is over                                                                                             |
| `get_active_invaders` | `env`                 | Returns number of invaders still active on screen                                                                            |
| `get_entity_count`    | `env`                 | Returns total ECS entity count for debugging/inspection                                                                      |
