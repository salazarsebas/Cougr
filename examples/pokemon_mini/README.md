# Pokémon Mini

A Pokémon-style mini game demonstrating on-chain mechanics like deterministic movement, encounters, and turn-based battles using the `cougr-core` ECS framework on the Stellar blockchain via Soroban.

# ECS Architecture
Components

`Position` — Stores `x, y` coordinates of entities (player or creatures)

`Direction` — Stores facing direction of the player

`Creature` — Stores stats like species, level, HP, attack, and defense

`BattleState` — Tracks current battle: turn, player/enemy HP, result

`GameState` — Tracks player stats: move count, wins, losses, escapes, battle state

Systems

`init_player()` — Creates the player entity with spawn position and starter creature

`move_player()` — Handles grid-based movement, collision detection, and encounter triggering

`start_battle()` — Initializes a battle when an encounter occurs

`process_battle_action()` — Executes player actions (Attack, Defend, Run) and resolves battle outcomes

`get_player_position() / get_player_creature()` — Query functions for player and creature state

`update_player_creature()` — Updates creature stats after battle (e.g., healing on win)

# Build & Test
```bash
cargo build
cargo test
stellar contract build
```

# Deploy to Testnet
```bash
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/pokemon_mini.wasm --source <secret-key> --network testnet
```

# Contract API
| Function             | Parameters            | Description                                                                                                   |
| -------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------- |
| `init_player`        | `env`                 | Initializes a new player at spawn with starter creature                                                       |
| `get_player_state`   | `env`                 | Returns (x, y, move_count, in_battle, HP)                                                                     |
| `get_creature_stats` | `env`                 | Returns creature stats (species_id, level, hp, max_hp, atk, def)                                              |
| `get_tile`           | `x: i32, y: i32`      | Returns tile type at specified coordinates (0: Grass, 1: Wall, 2: Water, 3: TallGrass, 4: Spawn)              |
| `get_map_size`       | —                     | Returns map dimensions `(width, height)`                                                                      |
| `get_battle_stats`   | `env`                 | Returns (wins, losses, escapes)                                                                               |
| `move_player`        | `env, direction: u32` | Moves player; direction 0=Up,1=Down,2=Left,3=Right; returns movement status (0=blocked, 1=moved, 2=encounter) |
| `get_battle_state`   | `env`                 | Returns (in_battle, player_hp, enemy_hp, turn, result)                                                        |
| `battle_action`      | `env, action: u32`    | Executes battle action (0=Attack,1=Defend,2=Run); returns action result (0=invalid,1=ongoing,2=finished)      |

