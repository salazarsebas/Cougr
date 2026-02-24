# Bomberman

A classic grid-based Bomberman game demonstrating on-chain game mechanics using `cougr-core` ECS for Soroban.

The contract showcases:

-Component-based architecture

-Persistent game state management

-Bomb placement, explosions, and player movement logic

-Efficient queries for scores and game-over conditions

# ECS Architecture
Components

`PlayerComponent` — stores each player's ID, position (x, y), lives, bomb capacity, and score.

`BombComponent` — stores bomb position, timer until explosion, explosion power, and owner player ID.

`ExplosionComponent` — tracks ongoing explosions, their positions, and timers.

`GridComponent` — stores the game grid cells (walls, destructible blocks, power-ups).

`GameStateComponent` — tracks current game tick, whether the game is over, and the winner.

Systems / Functions

`init_game(env)` — initializes the grid, players, bombs, explosions, and game state using cougr-core ECS.

`move_player(env, player_id, direction)` — moves a player (0=up, 1=right, 2=down, 3=left) while validating walkable tiles.

`place_bomb(env, player_id)` — places a bomb at the player's position.

`update_tick(env)` — advances the game tick, decrements timers, triggers explosions, handles collisions, and updates game state.

`get_score(env, player_id)` — retrieves the score for a specific player.

`check_game_over(env)` — checks if the game is over and identifies the winner.

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/bomberman.wasm --source <secret-key> --network testnet
```
# Contract API
| Function          | Parameters                                 | Description                                                                    |
| ----------------- | ------------------------------------------ | ------------------------------------------------------------------------------ |
| `init_game`       | `env: Env`                                 | Initializes the Bomberman game world and ECS components.                       |
| `move_player`     | `env: Env, player_id: u32, direction: u32` | Moves a player in the specified direction (0=up, 1=right, 2=down, 3=left).     |
| `place_bomb`      | `env: Env, player_id: u32`                 | Places a bomb at the player's current position.                                |
| `update_tick`     | `env: Env`                                 | Advances the game tick; updates bombs, explosions, collisions, and game state. |
| `get_score`       | `env: Env, player_id: u32`                 | Returns the current score for the specified player.                            |
| `check_game_over` | `env: Env`                                 | Returns the game-over status and winner if any.                                |
| `hello`           | `env: Env, to: Symbol`                     | Test function returning the symbol passed (example/demo).                      |


