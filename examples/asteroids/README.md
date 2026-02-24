# Asteroids On-Chain Game

Demonstrates an Asteroids-style game implemented on-chain using `cougr-core` ECS framework on the Stellar blockchain. Includes ship movement, shooting bullets, and asteroid collisions.

# ECS Architecture
Components

`Vec2` — 2D position and velocity (x and y)

`Ship` — Ship’s position, velocity, and rotation

`Asteroid` — Asteroid’s position, velocity, and size

`Bullet` — Bullet’s position, velocity, and TTL (time to live)

`GameState` — Stores current ship, bullets, asteroids, score, lives, and game-over status

Systems

`update_tick()` — Updates positions of ship, bullets, and asteroids; handles collisions; updates score and game-over state

`thrust_ship()` — Accelerates the ship in its current rotation

`rotate_ship(delta_steps)` — Rotates the ship clockwise/counterclockwise

`shoot()` — Fires a bullet if max bullets not reached

`cougr_smoke()` — Test function verifying Cougr-Core ECS integration

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/asteroids.wasm \
  --source <secret-key> \
  --network testnet
```

# Contract API
| Function          | Parameters        | Description                                                              |
| ----------------- | ----------------- | ------------------------------------------------------------------------ |
| `init_game`       | —                 | Initializes ship, asteroids, bullets, score, lives, and game-over state. |
| `thrust_ship`     | —                 | Accelerates the ship in its current direction.                           |
| `rotate_ship`     | `delta_steps:i32` | Rotates the ship clockwise or counter-clockwise.                         |
| `shoot`           | —                 | Fires a bullet (max 32 bullets, TTL 50).                                 |
| `update_tick`     | —                 | Moves ship, bullets, and asteroids; handles collisions and scoring.      |
| `get_score`       | —                 | Returns the current score.                                               |
| `check_game_over` | —                 | Returns `true` if the game is over.                                      |
| `cougr_smoke`     | —                 | Test function verifying Cougr-Core ECS integration.                      |


