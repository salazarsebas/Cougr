# Pong

A classic Pong game demonstrating on-chain mechanics using the `cougr-core` ECS framework on the Stellar blockchain via Soroban. The game showcases modular component and system patterns for paddles, ball physics, collisions, and scoring.

# ECS Architecture

Components

`PaddleComponent` — Stores player ID and paddle y-position

`BallComponent` — Stores ball position `(x, y)` and velocity `(vx, vy)`

`ScoreComponent` — Stores player scores and game active state

`ECSWorldState` — Aggregates all components into a single world entity

`GameState` — External representation of the game state for clients

Systems

`physics_system()` — Updates the ball’s position based on velocity

`collision_system()` — Handles collisions with walls and paddles

`scoring_system()` — Updates player scores, resets ball, and checks win conditions

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/pong.wasm --source <secret-key> --network testnet
```

# Contract API
| Function         | Parameters                         | Description                                                                          |
| ---------------- | ---------------------------------- | ------------------------------------------------------------------------------------ |
| `init_game`      | `env`                              | Initializes a new game with paddles, ball, and score using ECS pattern               |
| `move_paddle`    | `env, player: u32, direction: i32` | Moves the specified player’s paddle up or down; direction positive=down, negative=up |
| `update_tick`    | `env`                              | Advances the game by one tick: moves ball, checks collisions, updates scores         |
| `get_game_state` | `env`                              | Returns the current game state (paddles, ball, scores, game active)                  |
| `reset_game`     | `env`                              | Resets the game to initial state                                                     |

